use std::sync::Arc;

use async_trait::async_trait;
use rdkafka::producer::FutureProducer;

use crate::application::ports::{AuthEvent, AuthEventPublisher};
use crate::config::config::AppConfig;

use super::kafka_client::base_client_config;
use super::kafka_send::send_json_bytes;

pub struct KafkaAuthEventPublisher {
    producer: Arc<FutureProducer>,
    topic: String,
}

impl KafkaAuthEventPublisher {
    pub fn try_new(config: &AppConfig) -> Result<Self, rdkafka::error::KafkaError> {
        let mut client = base_client_config(config, "producer");
        client
            .set("message.timeout.ms", "10000")
            .set("acks", "all")
            .set("enable.idempotence", "true");

        let producer: FutureProducer = client.create()?;
        Ok(Self {
            producer: Arc::new(producer),
            topic: config.kafka_topic_auth_events.clone(),
        })
    }
}

#[async_trait]
impl AuthEventPublisher for KafkaAuthEventPublisher {
    async fn publish(&self, event: AuthEvent) -> Result<(), String> {
        let key = event.partition_key();
        let payload =
            serde_json::to_string(&event).map_err(|e| format!("serialize auth event: {e}"))?;

        send_json_bytes(&self.producer, &self.topic, &key, payload.as_bytes()).await
    }
}
