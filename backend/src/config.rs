use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub ro_database_url: String,
    pub jwt_secret: String,
    pub claude_bin: String,
    pub bind_addr: String,
    pub cors_origins: Vec<String>,
    pub setu_client_id: Option<String>,
    pub setu_client_secret: Option<String>,
    pub setu_product_instance_id: Option<String>,
    pub setu_base_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .context("JWT_SECRET not set")?;

        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET is too short ({} chars, minimum 32). Generate one with: openssl rand -hex 32", jwt_secret.len());
        }
        if jwt_secret.to_lowercase().contains("change-me") {
            anyhow::bail!("JWT_SECRET is a placeholder value. Generate a real secret with: openssl rand -hex 32");
        }

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL not set")?,
            ro_database_url: std::env::var("RO_DATABASE_URL")
                .context("RO_DATABASE_URL not set")?,
            jwt_secret,
            claude_bin: std::env::var("CLAUDE_BIN")
                .unwrap_or_else(|_| "claude".to_string()),
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8090".to_string()),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            setu_client_id: std::env::var("SETU_CLIENT_ID").ok(),
            setu_client_secret: std::env::var("SETU_CLIENT_SECRET").ok(),
            setu_product_instance_id: std::env::var("SETU_PRODUCT_INSTANCE_ID").ok(),
            setu_base_url: std::env::var("SETU_BASE_URL")
                .unwrap_or_else(|_| "https://qa.setu.co/api/v2".to_string()),
        })
    }
}
