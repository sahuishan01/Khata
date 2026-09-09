use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{fmt, EnvFilter};

use khata::{
    accounts, auth, budgets, categories, chat, config, db, goals, ingest, portfolio, reports, rules,
    subscriptions, txns, AppState,
};

async fn security_headers_mw(
    req: Request,
    next: Next,
) -> Response {
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    resp.headers_mut().insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    resp.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    resp.headers_mut().insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    resp.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https://api.anthropic.com;",
        ),
    );
    resp
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try ../.env first (when running `cargo run` from the backend/ subdir),
    // then .env in the current dir so a local override still wins.
    dotenvy::from_path("../.env").ok();
    dotenvy::dotenv().ok();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Pin the rustls crypto provider process-wide (used by the IMAP client).
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();

    let cfg = Arc::new(config::Config::from_env()?);

    match khata::ingest::pdf::extract::init(cfg.pdfium_lib_path.as_deref()) {
        Ok(()) => tracing::info!("pdfium: loaded"),
        Err(e) => tracing::warn!("{e} — PDF parsing will use the fallback parser"),
    }
    let db = db::make_pool(&cfg.database_url).await?;
    let db_ro = db::make_pool(&cfg.ro_database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        db_ro,
        config: cfg.clone(),
        chat_ratelimit: Arc::new(Mutex::new(HashMap::new())),
        login_attempts: Arc::new(Mutex::new(HashMap::new())),
    };

    // Background Gmail statement ingestion poll loop.
    ingest::email::spawn_worker(state.clone());

    let cors = {
        let origins: Vec<axum::http::HeaderValue> = cfg
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        if origins.is_empty() {
            // Fail-closed: no cross-origin requests allowed
            CorsLayer::new()
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
        .nest("/api/accounts", accounts::router())
        .nest("/api/ingest", ingest::router())
        .nest("/api/rules", rules::router())
        .nest("/api/budgets", budgets::router())
        .nest("/api/categories", categories::router())
        .nest("/api/portfolio", portfolio::router())
        .nest("/api/subscriptions", subscriptions::router())
        .nest("/api/goals", goals::router())
        .nest("/api/reports", reports::router())
        .nest("/api/txns", txns::router())
        .nest("/api/chat", chat::router())
        .layer(middleware::from_fn(security_headers_mw))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    tracing::info!("listening on {}", cfg.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
