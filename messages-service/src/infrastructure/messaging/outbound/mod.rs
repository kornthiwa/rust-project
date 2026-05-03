//! Kafka adapters for [`crate::application::ports::MessageEventPublisher`].
//! Shared connection helpers stay at [`super::kafka_client`].

mod kafka_producer;
mod kafka_send;
mod noop_publisher;

pub use kafka_producer::KafkaMessageEventPublisher;
pub use noop_publisher::NoopMessageEventPublisher;
