use std::env;
use dotenv::dotenv;

pub struct AppConfig {
    pub port_config: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        let host = env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1"));
        let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            String::from("")
        });
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| String::from("please-change-this-secret"));
        let jwt_expiration_seconds = env::var("JWT_EXPIRATION_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(3600);

        Self {
            port_config: format!("{}:{}", host, port),
            database_url,
            jwt_secret,
            jwt_expiration_seconds,
        }
    }
}
