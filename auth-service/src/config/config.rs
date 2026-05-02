use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnv(&'static str),
    InvalidEnv(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnv(name) => write!(f, "missing required env var: {name}"),
            ConfigError::InvalidEnv(name) => write!(f, "invalid value for env var: {name}"),
        }
    }
}
impl std::error::Error for ConfigError {}

fn required_trimmed(name: &'static str) -> Result<String, ConfigError> {
    let value = env::var(name).map_err(|_| ConfigError::MissingEnv(name))?;
    if value.trim().is_empty() {
        return Err(ConfigError::InvalidEnv(name));
    }
    Ok(value)
}

fn default_trimmed(name: &'static str, default: &'static str) -> Result<String, ConfigError> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    if value.trim().is_empty() {
        return Err(ConfigError::InvalidEnv(name));
    }
    Ok(value)
}

fn parse_i64_default(name: &'static str, default: i64) -> Result<i64, ConfigError> {
    match env::var(name) {
        Ok(raw) => raw
            .parse::<i64>()
            .map_err(|_| ConfigError::InvalidEnv(name)),
        Err(_) => Ok(default),
    }
}

fn parse_u16_default(name: &'static str, default: u16) -> Result<u16, ConfigError> {
    match env::var(name) {
        Ok(raw) => raw
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidEnv(name)),
        Err(_) => Ok(default),
    }
}

pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let host = default_trimmed("HOST", "127.0.0.1")?;
        let port = parse_u16_default("PORT", 3000)?;
        let database_url = required_trimmed("DATABASE_URL")?;
        let jwt_secret = required_trimmed("JWT_SECRET")?;
        let jwt_expiration_seconds = parse_i64_default("JWT_EXPIRATION_SECONDS", 3600)?;
        if jwt_expiration_seconds <= 0 {
            return Err(ConfigError::InvalidEnv("JWT_EXPIRATION_SECONDS"));
        }

        Ok(Self {
            host,
            port,
            database_url,
            jwt_secret,
            jwt_expiration_seconds,
        })
    }

    pub fn port_config(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
