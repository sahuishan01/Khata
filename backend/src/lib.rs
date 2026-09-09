pub mod accounts;
pub mod audit;
pub mod auth;
pub mod budgets;
pub mod categories;
pub mod chat;
pub mod config;
pub mod db;
pub mod error;
pub mod goals;
pub mod ingest;
pub mod portfolio;
pub mod reports;
pub mod rules;
pub mod subscriptions;
pub mod txns;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub db_ro: sqlx::PgPool,
    pub config: Arc<config::Config>,
    pub chat_ratelimit: Arc<Mutex<HashMap<Uuid, Instant>>>,
    pub login_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}
