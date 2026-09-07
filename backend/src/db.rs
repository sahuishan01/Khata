use sqlx::{postgres::PgPoolOptions, PgPool, Executor};
use uuid::Uuid;

pub async fn make_pool(url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?)
}

/// Set the RLS user context for the current transaction.
/// Must be called inside a transaction before querying user-scoped tables.
pub async fn set_current_user<'c, E>(executor: E, user_id: Uuid) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = sqlx::Postgres>,
{
    // set_config(..., is_local = true) is the transaction-scoped equivalent of
    // `SET LOCAL`, but unlike `SET LOCAL` it accepts a bind parameter — so the
    // user id never touches the SQL string. RLS reads it back via
    // current_setting('app.current_user_id', true)::uuid.
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(executor)
        .await?;
    Ok(())
}
