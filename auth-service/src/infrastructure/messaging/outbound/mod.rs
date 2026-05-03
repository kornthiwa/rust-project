//! Kafka adapters for [`crate::application::ports::AuthEventPublisher`] (outbound / produce path).
//! Shared connection helpers stay at [`super::kafka_client`].

mod kafka_producer;
mod kafka_send;
mod noop_publisher;

pub use kafka_producer::KafkaAuthEventPublisher;
pub use noop_publisher::NoopAuthEventPublisher;
