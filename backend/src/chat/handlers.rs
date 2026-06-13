use axum::{extract::State, Json};
use std::time::Instant;
use uuid::Uuid;

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};

use super::{claude_cli, models::*, predefined, sql_validator::validate_select_sql};

pub async fn ask_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<AskReq>,
) -> Result<Json<AskResponse>, AppError> {
    if req.question.trim().is_empty() {
        return Err(AppError::BadRequest("question is empty".into()));
    }

    // ── Rate limiting: one call per user per 30 seconds ──────────────────
    {
        let mut ratelimit = state.chat_ratelimit.lock().unwrap();
        let now = Instant::now();

        // Clean up stale entries older than 5 minutes
        ratelimit.retain(|_, last| now.duration_since(*last).as_secs() < 300);

        if let Some(last) = ratelimit.get(&user_id) {
            if now.duration_since(*last).as_secs() < 30 {
                return Err(AppError::TooManyRequests);
            }
        }
        ratelimit.insert(user_id, now);
    }

    // Fetch user's categories — used for predefined category-spend matching
    // and as context when we do fall back to Claude.
    let categories: Vec<String> = {
        let mut tx = state.db.begin().await?;
        sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
            .execute(&mut *tx)
            .await?;
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT category FROM transactions WHERE user_id = $1 ORDER BY category",
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(|(c,)| c).collect()
    };

    let (sql, answer, row_count) =
        if let Some(pre) = predefined::match_question(&req.question, &categories) {
            // ── Predefined path: no Claude call at all ────────────────────────
            let rows_json = execute_safe(&state.db_ro, user_id, &pre.sql).await?;
            let rows: Vec<serde_json::Value> =
                serde_json::from_str(&rows_json).unwrap_or_default();
            let answer = predefined::format_predefined(pre.kind, &rows, &pre.label);
            (pre.sql, answer, rows.len())
        } else {
            // ── Claude fallback: one call to generate SQL ─────────────────────
            let sql_raw =
                claude_cli::generate_sql(&state.config.claude_bin, &req.question, &categories)
                    .await
                    .map_err(|e| AppError::BadRequest(format!("SQL generation failed: {e}")))?;

            let sql_gen: SqlGenOutput = serde_json::from_str(&sql_raw).map_err(|_| {
                AppError::BadRequest(format!("Unexpected SQL gen response: {sql_raw}"))
            })?;

            validate_select_sql(&sql_gen.sql)
                .map_err(|e| AppError::BadRequest(format!("Unsafe SQL rejected: {e}")))?;

            let rows_json = execute_safe(&state.db_ro, user_id, &sql_gen.sql).await?;
            let rows: Vec<serde_json::Value> =
                serde_json::from_str(&rows_json).unwrap_or_default();
            let answer = predefined::format_generic(&rows);
            (sql_gen.sql, answer, rows.len())
        };

    // Persist Q + A
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO chat_messages (user_id, role, content) VALUES ($1, 'user', $2)")
        .bind(user_id)
        .bind(&req.question)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO chat_messages (user_id, role, content, sql_used) VALUES ($1, 'assistant', $2, $3)",
    )
    .bind(user_id)
    .bind(&answer)
    .bind(&sql)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Audit log
    audit::log_audit(
        &state.db,
        user_id,
        "chat_query",
        Some(serde_json::json!({
            "question": &req.question,
            "sql": &sql,
            "row_count": row_count,
        })),
    ).await;

    Ok(Json(AskResponse { answer, sql_used: sql }))
}

pub async fn history_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<ChatMessage>>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;
    let msgs = sqlx::query_as::<_, ChatMessage>(
        "SELECT id, role, content, sql_used, created_at \
         FROM chat_messages WHERE user_id = $1 ORDER BY created_at ASC LIMIT 100",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(msgs))
}

async fn execute_safe(
    db_ro: &sqlx::PgPool,
    user_id: Uuid,
    sql: &str,
) -> Result<String, AppError> {
    let mut tx = db_ro.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL statement_timeout = '5000'")
        .execute(&mut *tx)
        .await?;

    // Wrap in a subselect so we can cap at 200 without conflicting with any
    // LIMIT already present in the inner SQL (e.g. predefined queries with LIMIT 5/10).
    let capped = format!("SELECT row_to_json(q) FROM (SELECT * FROM ({sql}) _raw LIMIT 200) q");
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as(&capped)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::BadRequest(format!("Query execution error: {e}")))?;

    tx.rollback().await?;

    let arr: Vec<serde_json::Value> = rows.into_iter().map(|(v,)| v).collect();
    Ok(serde_json::to_string(&arr).unwrap_or_else(|_| "[]".into()))
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::ingest::store::insert_statement;

    #[sqlx::test(migrations = "./migrations")]
    async fn rls_isolates_users(pool: PgPool) {
        let (uid_a,): (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email,password_hash) VALUES ('a@rls.com','x') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let (uid_b,): (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email,password_hash) VALUES ('b@rls.com','x') RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let sid_a = insert_statement(&pool, uid_a, "HDFC", "a.csv", "sha_a", 1)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        sqlx::query(&format!("SET LOCAL app.current_user_id = '{uid_a}'"))
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO transactions \
             (user_id,statement_id,bank,account_label,txn_date,value_date,description,raw_description,amount,direction,fingerprint) \
             VALUES ($1,$2,'HDFC','',NOW(),NOW(),'test','test',100,'debit','fp_rls_test')",
        )
        .bind(uid_a)
        .bind(sid_a)
        .execute(&mut *tx)
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let mut tx_b = pool.begin().await.unwrap();
        sqlx::query(&format!("SET LOCAL app.current_user_id = '{uid_b}'"))
            .execute(&mut *tx_b)
            .await
            .unwrap();
        let rows: Vec<(i64,)> = sqlx::query_as("SELECT COUNT(*)::bigint FROM transactions")
            .fetch_all(&mut *tx_b)
            .await
            .unwrap();
        tx_b.rollback().await.unwrap();

        assert_eq!(rows[0].0, 0, "user B must not see user A's transactions via RLS");
    }
}
