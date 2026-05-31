use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub ro_database_url: String,
    pub jwt_secret: String,
    pub claude_bin: String,
    pub bind_addr: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL not set")?,
            ro_database_url: std::env::var("RO_DATABASE_URL")
                .context("RO_DATABASE_URL not set")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET not set")?,
            claude_bin: std::env::var("CLAUDE_BIN")
                .unwrap_or_else(|_| "claude".to_string()),
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}
