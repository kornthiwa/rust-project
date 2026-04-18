pub struct AppConfig {
    pub database_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL is not set. Define it in your environment or .env file.");
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET is not set. Define it in your environment or .env file.");

        Self {
            database_url,
            bind_addr: format!("0.0.0.0:{port}"),
            jwt_secret,
        }
    }
}
