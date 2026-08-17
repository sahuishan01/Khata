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
    // Uuid::Display only produces hex + hyphens, so this is injection-safe.
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(executor)
        .await?;
    Ok(())
}
