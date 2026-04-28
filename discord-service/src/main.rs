mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::infrastructure::mongo::MongoPool;
use crate::presentation::discord::bootstrap;
use dotenv::dotenv;
use std::error::Error;

fn ensure_env() -> Result<(), Box<dyn Error>> {
    for key in ["DISCORD_TOKEN", "MONGO_URI", "MONGO_DB_NAME"] {
        std::env::var(key).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("ไม่พบ {key} ใน environment"),
            )
        })?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    ensure_env()?;

    let mongo_pool = MongoPool::connect_from_env().await?;
    let app = bootstrap::build_application(mongo_pool)
        .map_err(|e| std::io::Error::other(format!("bootstrap failed: {e}")))?;
    bootstrap::run_discord_bot(app).await?;

    Ok(())
}
