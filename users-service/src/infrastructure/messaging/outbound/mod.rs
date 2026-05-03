//! Kafka adapters for [`crate::application::ports::UserEventPublisher`].

mod kafka_producer;
mod kafka_send;
mod noop_publisher;

pub use kafka_producer::KafkaUserEventPublisher;
pub use noop_publisher::NoopUserEventPublisher;
