//! Shared Kafka produce helper — any bounded context can call this with its own topic + key + JSON
//! without touching `AuthEventPublisher` or auth-specific adapters.

use std::time::Duration;

use rdkafka::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

pub(in crate::infrastructure::messaging) async fn send_json_bytes(
    producer: &FutureProducer,
    topic: &str,
    partition_key: &str,
    payload: &[u8],
) -> Result<(), String> {
    producer
        .send(
            FutureRecord::to(topic)
                .key(partition_key)
                .payload(payload),
            Timeout::After(Duration::from_secs(5)),
        )
        .await
        .map_err(|(e, owned)| {
            tracing::warn!(
                error = %e,
                partition = owned.partition(),
                "kafka produce failed"
            );
            e.to_string()
        })?;
    Ok(())
}
