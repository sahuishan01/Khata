mod auth;
mod chat;
mod config;
mod db;
mod error;
mod ingest;
mod txns;

use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub db_ro: sqlx::PgPool,
    pub config: Arc<config::Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try ../.env first (when running `cargo run` from the backend/ subdir),
    // then .env in the current dir so a local override still wins.
    dotenvy::from_path("../.env").ok();
    dotenvy::dotenv().ok();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let cfg = Arc::new(config::Config::from_env()?);
    let db = db::make_pool(&cfg.database_url).await?;
    let db_ro = db::make_pool(&cfg.ro_database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        db_ro,
        config: cfg.clone(),
    };

    let cors = {
        let origins: Vec<axum::http::HeaderValue> = cfg
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        if origins.is_empty() {
            CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
        } else {
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/auth", auth::router())
        .nest("/api/ingest", ingest::router())
        .nest("/api/txns", txns::router())
        .nest("/api/chat", chat::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    tracing::info!("listening on {}", cfg.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
