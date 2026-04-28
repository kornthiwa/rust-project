use std::env;
use dotenv::dotenv;

pub struct AppConfig {
    pub port_config: String,
    pub database_url: String,
}

impl AppConfig {
    
    pub fn from_env() -> Self {
        dotenv().ok();

        let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| String::from("postgres://postgres:postgres@localhost:5432/auth_service"));

        Self {
            port_config: format!("127.0.0.1:{}", port),
            database_url,
        }
    }
}
