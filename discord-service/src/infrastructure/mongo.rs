use mongodb::{Client, Database, options::ClientOptions};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct MongoPool {
    client: Client,
    db_name: String,
}

impl MongoPool {
    pub async fn connect_from_env() -> Result<Self, mongodb::error::Error> {
        let mongo_uri = std::env::var("MONGO_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let db_name =
            std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "discord_bot".to_string());

        let mut options = ClientOptions::parse(&mongo_uri).await?;
        options.max_pool_size = Some(100);
        options.min_pool_size = Some(5);
        options.max_idle_time = Some(Duration::from_secs(60));
        options.server_selection_timeout = Some(Duration::from_secs(5));
        let client = Client::with_options(options)?;

        client.list_database_names().await?;
        Ok(Self { client, db_name })
    }

    pub fn database(&self) -> Database {
        self.client.database(&self.db_name)
    }
}
