//! Background poll loop: every `EMAIL_SYNC_POLL_SECS`, run a sync for each
//! `sync_enabled` user that has not synced within the interval.

use std::time::Duration;

use crate::AppState;

use super::run::{sync_user, Trigger};

pub fn spawn(state: AppState) {
    let secs = state.config.email_sync_poll_secs;
    if secs == 0 {
        tracing::info!("email sync poll loop disabled (EMAIL_SYNC_POLL_SECS=0)");
        return;
    }
    tokio::spawn(async move {
        // Small initial delay so startup isn't contended.
        tokio::time::sleep(Duration::from_secs(15)).await;
        let interval = Duration::from_secs(secs);
        loop {
            if let Err(e) = tick(&state).await {
                tracing::warn!("email sync poll tick failed: {e:#}");
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn tick(state: &AppState) -> anyhow::Result<()> {
    // Users due for a sync: enabled, and (never synced OR synced > interval ago),
    // and without a run started in the last interval.
    let secs = state.config.email_sync_poll_secs as i64;
    let due: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.user_id FROM user_email_configs c \
         WHERE c.sync_enabled = true \
           AND (c.last_synced_at IS NULL OR c.last_synced_at < now() - ($1 || ' seconds')::interval) \
           AND NOT EXISTS ( \
               SELECT 1 FROM email_sync_runs r \
               WHERE r.user_id = c.user_id AND r.status = 'running' \
           )",
    )
    .bind(secs.to_string())
    .fetch_all(&state.db)
    .await?;

    for (user_id,) in due {
        match sync_user(state.clone(), user_id, Trigger::Poll).await {
            Ok(run_id) => tracing::info!(%user_id, %run_id, "email sync run started (poll)"),
            Err(e) => tracing::debug!(%user_id, "email sync poll skipped: {e:#}"),
        }
    }
    Ok(())
}
