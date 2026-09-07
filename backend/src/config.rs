use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub ro_database_url: String,
    pub jwt_secret: String,
    pub claude_bin: String,
    pub bind_addr: String,
    pub cors_origins: Vec<String>,
    /// Whether the `Secure` attribute is set on the `khata_token` auth cookie.
    /// Defaults to `true`. Set `COOKIE_SECURE=false` ONLY for local HTTP
    /// development: the bind address is not a reliable signal of transport
    /// security because the documented production topology binds the backend to
    /// loopback behind a TLS-terminating reverse proxy.
    pub cookie_secure: bool,
    /// Allow the unauthenticated first-run `POST /api/auth/setup` endpoint even
    /// when the server is not bound to a loopback address. Defaults to `false`
    /// so a backend accidentally (or deliberately) exposed on a public
    /// interface cannot have its sole admin account claimed by a stranger.
    pub allow_remote_setup: bool,
    /// Email ingestion background poll interval, seconds. `0` disables the poll
    /// loop (manual `POST /api/ingest/email/sync` still works). Default 900.
    pub email_sync_poll_secs: u64,
    /// Max messages processed per email sync run. Default 200.
    pub email_sync_max_messages: usize,
    /// Per-attachment size cap for email sync, bytes. Default 15 MiB.
    pub email_sync_max_attach_bytes: usize,
}

/// Parse an unsigned-integer env var; unset or unparseable falls back to `default`.
fn env_uint<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// Parse a boolean-ish environment variable. Truthy values: `1`, `true`, `yes`,
/// `on` (case-insensitive). Anything else is false; an unset var uses `default`.
fn env_flag(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => default,
    }
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
            cookie_secure: env_flag("COOKIE_SECURE", true),
            allow_remote_setup: env_flag("KHATA_ALLOW_REMOTE_SETUP", false),
            email_sync_poll_secs: env_uint("EMAIL_SYNC_POLL_SECS", 900),
            email_sync_max_messages: env_uint("EMAIL_SYNC_MAX_MESSAGES", 200),
            email_sync_max_attach_bytes: env_uint("EMAIL_SYNC_MAX_ATTACH_BYTES", 15 * 1024 * 1024),
        })
    }

    /// True when `bind_addr` refers to a loopback host (`127.0.0.0/8`, `::1`) or
    /// `localhost`. Used to gate the unauthenticated first-run setup endpoint.
    pub fn bind_is_loopback(&self) -> bool {
        let addr = self.bind_addr.trim();
        let host = if let Some(rest) = addr.strip_prefix('[') {
            // Bracketed IPv6: `[::1]:8090` -> `::1`
            rest.split(']').next().unwrap_or(rest)
        } else if addr.matches(':').count() == 1 {
            // `127.0.0.1:8090` -> `127.0.0.1`
            addr.split(':').next().unwrap_or(addr)
        } else {
            // Bare host or bare IPv6 with no port
            addr
        };
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .map(|ip| ip.is_loopback())
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with_bind(bind: &str) -> Config {
        Config {
            database_url: String::new(),
            ro_database_url: String::new(),
            jwt_secret: String::new(),
            claude_bin: String::new(),
            bind_addr: bind.into(),
            cors_origins: vec![],
            cookie_secure: true,
            allow_remote_setup: false,
            email_sync_poll_secs: 0,
            email_sync_max_messages: 200,
            email_sync_max_attach_bytes: 15 * 1024 * 1024,
        }
    }

    #[test]
    fn loopback_bind_detection() {
        assert!(cfg_with_bind("127.0.0.1:8090").bind_is_loopback());
        assert!(cfg_with_bind("localhost:8090").bind_is_loopback());
        assert!(cfg_with_bind("[::1]:8090").bind_is_loopback());
        assert!(cfg_with_bind("127.0.0.5:1").bind_is_loopback());
        assert!(cfg_with_bind("::1").bind_is_loopback());

        assert!(!cfg_with_bind("0.0.0.0:8080").bind_is_loopback());
        assert!(!cfg_with_bind("192.168.1.10:8080").bind_is_loopback());
        assert!(!cfg_with_bind("10.0.0.2:80").bind_is_loopback());
    }

    #[test]
    fn env_flag_parsing() {
        // Use a unique key so parallel tests don't collide.
        let key = "KHATA_TEST_ENV_FLAG_XYZ";
        std::env::remove_var(key);
        assert!(env_flag(key, true));
        assert!(!env_flag(key, false));
        std::env::set_var(key, "false");
        assert!(!env_flag(key, true));
        std::env::set_var(key, "1");
        assert!(env_flag(key, false));
        std::env::remove_var(key);
    }
}
