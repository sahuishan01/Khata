use sqlx::PgPool;
use uuid::Uuid;

pub async fn log_audit(
    db: &PgPool,
    user_id: Uuid,
    action: &str,
    details: Option<serde_json::Value>,
) {
    let details = details.map(|d| d.to_string());
    if let Err(e) = sqlx::query(
        "INSERT INTO audit_log (user_id, action, details) VALUES ($1, $2, $3::jsonb)",
    )
    .bind(user_id)
    .bind(action)
    .bind(details)
    .execute(db)
    .await
    {
        tracing::error!("audit log failed: {e}");
    }
}
