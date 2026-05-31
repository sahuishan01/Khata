use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn make_pool(url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?)
}
