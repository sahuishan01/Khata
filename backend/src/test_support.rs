//! Shared fixtures for `#[sqlx::test]` handler tests.

use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppState;

pub fn app_state(pool: &PgPool) -> AppState {
    AppState {
        db: pool.clone(),
        db_ro: pool.clone(),
        config: Arc::new(crate::config::Config {
            database_url: String::new(),
            ro_database_url: String::new(),
            jwt_secret: "test-secret-32-chars-min-aaaaaaaaaa".into(),
            claude_bin: "claude".into(),
            bind_addr: "127.0.0.1:0".into(),
            cors_origins: vec![],
            cookie_secure: true,
            allow_remote_setup: false,
            email_sync_poll_secs: 0,
            email_sync_max_messages: 200,
            email_sync_max_attach_bytes: 15 * 1024 * 1024,
            pdfium_lib_path: None,
        }),
        chat_ratelimit: Arc::new(Mutex::new(std::collections::HashMap::new())),
        login_attempts: Arc::new(Mutex::new(std::collections::HashMap::new())),
    }
}

pub async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1,$2,'x')")
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(pool)
        .await
        .unwrap();
    id
}

/// Insert one transaction under the user's RLS context and return its id.
pub async fn seed_txn(
    pool: &PgPool,
    user_id: Uuid,
    bank: &str,
    description: &str,
    direction: &str,
    amount: f64,
    date: NaiveDate,
    category: &str,
) -> Uuid {
    let stmt = crate::ingest::store::insert_statement(
        pool,
        user_id,
        bank,
        "seed.csv",
        &Uuid::new_v4().to_string(),
        1,
    )
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    crate::db::set_current_user(&mut *tx, user_id).await.unwrap();
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO transactions (user_id, statement_id, bank, txn_date, value_date, description, \
         raw_description, amount, direction, fingerprint, category) \
         VALUES ($1,$2,$3,$4,$4,$5,$5,$6,$7,$8,$9) RETURNING id",
    )
    .bind(user_id)
    .bind(stmt)
    .bind(bank)
    .bind(date)
    .bind(description)
    .bind(amount)
    .bind(direction)
    .bind(Uuid::new_v4().to_string())
    .bind(category)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

/// Read one column of a transaction under the user's RLS context.
pub async fn txn_field<T>(pool: &PgPool, user_id: Uuid, id: Uuid, col: &str) -> T
where
    T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Unpin,
{
    let mut tx = pool.begin().await.unwrap();
    crate::db::set_current_user(&mut *tx, user_id).await.unwrap();
    let (v,): (T,) = sqlx::query_as(&format!("SELECT {col} FROM transactions WHERE id = $1"))
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    v
}
