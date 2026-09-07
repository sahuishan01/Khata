//! Gmail statement ingestion: per-user IMAP config, a background poll worker,
//! and per-run history.

pub mod config;
pub mod handlers_runs;
pub mod imap;
pub mod mime;
pub mod pdf_decrypt;
pub mod run;
pub mod worker;

pub use config::{
    delete_email_config_handler, get_email_config_handler, save_email_config_handler,
};
pub use handlers_runs::{latest_run_handler, list_runs_handler, trigger_email_sync_handler};

/// Start the background poll loop. No-op when `EMAIL_SYNC_POLL_SECS=0`.
pub fn spawn_worker(state: crate::AppState) {
    worker::spawn(state);
}
