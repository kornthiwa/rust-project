use rdkafka::ClientConfig;
use uuid::Uuid;

use crate::config::config::AppConfig;

/// Shared librdkafka settings: bootstrap + unique `client.id` (visible in broker logs).
pub(crate) fn base_client_config(config: &AppConfig, role: &'static str) -> ClientConfig {
    let mut c = ClientConfig::new();
    c.set("bootstrap.servers", &config.kafka_bootstrap_servers);
    c.set(
        "client.id",
        format!("auth-service-{}-{}", role, Uuid::new_v4()),
    );
    c
}
