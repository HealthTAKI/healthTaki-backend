use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in_seconds: i64,
    pub auth_challenge_ttl_seconds: i64,
    pub port: u16,
    pub cors_allowed_origin: String,
    pub stellar_network: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            jwt_secret: require_env("JWT_SECRET")?,
            jwt_expires_in_seconds: env_or("JWT_EXPIRES_IN_SECONDS", 60 * 60 * 24),
            auth_challenge_ttl_seconds: env_or("AUTH_CHALLENGE_TTL_SECONDS", 300),
            port: env_or("PORT", 8080),
            cors_allowed_origin: env::var("CORS_ALLOWED_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            stellar_network: env::var("STELLAR_NETWORK").unwrap_or_else(|_| "TESTNET".to_string()),
        })
    }
}

fn require_env(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("missing required env var {key}"))
}

fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
