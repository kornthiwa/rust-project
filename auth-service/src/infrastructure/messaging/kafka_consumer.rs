use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::config::config::AppConfig;

use super::kafka_client::base_client_config;

pub fn spawn_auth_event_consumer_if_enabled(config: &AppConfig) {
    if !config.kafka_enabled {
        return;
    }

    let topic = config.kafka_topic_auth_events.clone();
    let group = config.kafka_consumer_group.clone();

    let mut client = base_client_config(config, "consumer");
    client
        .set("group.id", &group)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "45000")
        .set("heartbeat.interval.ms", "3000");

    let consumer: StreamConsumer = match client.create() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to create kafka consumer");
            return;
        }
    };

    if let Err(e) = consumer.subscribe(&[topic.as_str()]) {
        tracing::error!(error = %e, "failed to subscribe kafka consumer");
        return;
    }

    tokio::spawn(async move {
        tracing::info!(topic = %topic, group = %group, "kafka consumer started for auth events");
        let mut stream = consumer.stream();
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    let body = msg
                        .payload()
                        .map(|p| String::from_utf8_lossy(p).into_owned())
                        .unwrap_or_default();
                    tracing::info!(
                        topic = msg.topic(),
                        partition = msg.partition(),
                        offset = msg.offset(),
                        payload = %body,
                        "kafka consumer: auth event received"
                    );
                }
                Err(e) => tracing::warn!(error = %e, "kafka stream error"),
            }
        }
        tracing::warn!("kafka consumer stream ended");
    });
}
