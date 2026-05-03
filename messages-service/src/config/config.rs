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
            Self::MissingEnv(name) => write!(f, "missing required env var: {name}"),
            Self::InvalidEnv(name) => write!(f, "invalid value for env var: {name}"),
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

fn parse_u16_default(name: &'static str, default: u16) -> Result<u16, ConfigError> {
    match env::var(name) {
        Ok(raw) => raw
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidEnv(name)),
        Err(_) => Ok(default),
    }
}

/// When unset, defaults to `true`. Set `false` / `0` / `no` / `off` to disable.
fn parse_kafka_enabled() -> bool {
    match env::var("KAFKA_ENABLED") {
        Ok(s) => {
            let t = s.trim().to_lowercase();
            if t.is_empty() {
                true
            } else {
                !matches!(t.as_str(), "false" | "0" | "no" | "off")
            }
        }
        Err(_) => true,
    }
}

pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub kafka_enabled: bool,
    pub kafka_bootstrap_servers: String,
    pub kafka_topic_message_events: String,
    /// Same default as auth-service: login/register events for optional cross-service consume.
    pub kafka_topic_auth_events: String,
    pub kafka_consumer_group: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        let host = default_trimmed("HOST", "127.0.0.1")?;
        let port = parse_u16_default("PORT", 3200)?;
        let database_url = required_trimmed("DATABASE_URL")?;
        let jwt_secret = required_trimmed("JWT_SECRET")?;

        let kafka_enabled = parse_kafka_enabled();
        let kafka_bootstrap_servers =
            default_trimmed("KAFKA_BOOTSTRAP_SERVERS", "127.0.0.1:9092")?;
        let kafka_topic_message_events =
            default_trimmed("KAFKA_TOPIC_MESSAGE_EVENTS", "messages.events")?;
        let kafka_topic_auth_events =
            default_trimmed("KAFKA_TOPIC_AUTH_EVENTS", "auth.events")?;
        let kafka_consumer_group = default_trimmed("KAFKA_CONSUMER_GROUP", "messages-service")?;

        Ok(Self {
            host,
            port,
            database_url,
            jwt_secret,
            kafka_enabled,
            kafka_bootstrap_servers,
            kafka_topic_message_events,
            kafka_topic_auth_events,
            kafka_consumer_group,
        })
    }

    pub fn port_config(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
