use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::config::config::AppConfig;

use super::kafka_admin::{ensure_auth_events_topic, ensure_message_events_topic};
use super::kafka_client::base_client_config;

pub async fn spawn_message_event_consumer_if_enabled(config: &AppConfig) {
    if !config.kafka_enabled {
        return;
    }

    if let Err(e) = ensure_message_events_topic(config).await {
        tracing::error!(
            error = %e,
            "failed to ensure kafka topic for message events; consumer not started"
        );
        return;
    }

    if let Err(e) = ensure_auth_events_topic(config).await {
        tracing::error!(
            error = %e,
            "failed to ensure kafka topic for auth events; consumer not started"
        );
        return;
    }

    let message_topic = config.kafka_topic_message_events.clone();
    let auth_topic = config.kafka_topic_auth_events.clone();
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

    if let Err(e) = consumer.subscribe(&[message_topic.as_str(), auth_topic.as_str()]) {
        tracing::error!(error = %e, "failed to subscribe kafka consumer");
        return;
    }

    tokio::spawn(async move {
        tracing::info!(
            service = "messages-service",
            topics = ?[&message_topic, &auth_topic],
            group = %group,
            "kafka consumer started"
        );
        let mut stream = consumer.stream();
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    let body = msg
                        .payload()
                        .map(|p| String::from_utf8_lossy(p).into_owned())
                        .unwrap_or_default();
                    let t = msg.topic();
                    let kind = if t == message_topic.as_str() {
                        "message_event"
                    } else if t == auth_topic.as_str() {
                        "auth_event"
                    } else {
                        "unknown_topic"
                    };
                    tracing::info!(
                        topic = t,
                        kind,
                        partition = msg.partition(),
                        offset = msg.offset(),
                        payload = %body,
                        "kafka consumer event"
                    );
                }
                Err(e) => tracing::warn!(error = %e, "kafka stream error"),
            }
        }
        tracing::warn!("kafka consumer stream ended");
    });
}
