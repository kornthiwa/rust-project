use std::sync::Arc;

use async_trait::async_trait;
use rdkafka::producer::FutureProducer;

use crate::application::ports::{UserEvent, UserEventPublisher};
use crate::config::config::AppConfig;
use crate::infrastructure::messaging::kafka_client::base_client_config;

use super::kafka_send::send_json_bytes;

pub struct KafkaUserEventPublisher {
    producer: Arc<FutureProducer>,
    topic: String,
}

impl KafkaUserEventPublisher {
    pub fn try_new(config: &AppConfig) -> Result<Self, rdkafka::error::KafkaError> {
        let mut client = base_client_config(config, "producer");
        client
            .set("message.timeout.ms", "10000")
            .set("acks", "all")
            .set("enable.idempotence", "true");

        let producer: FutureProducer = client.create()?;
        Ok(Self {
            producer: Arc::new(producer),
            topic: config.kafka_topic_user_events.clone(),
        })
    }
}

#[async_trait]
impl UserEventPublisher for KafkaUserEventPublisher {
    async fn publish(&self, event: UserEvent) -> Result<(), String> {
        let key = event.partition_key();
        let payload =
            serde_json::to_string(&event).map_err(|e| format!("serialize user event: {e}"))?;

        send_json_bytes(&self.producer, &self.topic, &key, payload.as_bytes()).await
    }
}
