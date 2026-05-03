use std::sync::Arc;

use async_trait::async_trait;
use rdkafka::producer::FutureProducer;

use crate::application::ports::{MessageEvent, MessageEventPublisher};
use crate::config::config::AppConfig;

use super::kafka_client::base_client_config;
use super::kafka_send::send_json_bytes;

pub struct KafkaMessageEventPublisher {
    producer: Arc<FutureProducer>,
    topic: String,
}

impl KafkaMessageEventPublisher {
    pub fn try_new(config: &AppConfig) -> Result<Self, rdkafka::error::KafkaError> {
        let mut client = base_client_config(config, "producer");
        client
            .set("message.timeout.ms", "10000")
            .set("acks", "all")
            .set("enable.idempotence", "true");

        let producer: FutureProducer = client.create()?;
        Ok(Self {
            producer: Arc::new(producer),
            topic: config.kafka_topic_message_events.clone(),
        })
    }
}

#[async_trait]
impl MessageEventPublisher for KafkaMessageEventPublisher {
    async fn publish(&self, event: MessageEvent) -> Result<(), String> {
        let key = event.partition_key();
        let payload =
            serde_json::to_string(&event).map_err(|e| format!("serialize message event: {e}"))?;

        send_json_bytes(&self.producer, &self.topic, &key, payload.as_bytes()).await
    }
}
