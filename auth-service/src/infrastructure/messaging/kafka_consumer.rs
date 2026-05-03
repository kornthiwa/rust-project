use std::sync::Arc;

use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

use crate::application::ports::{AuthEvent, AuthEventInboundHandlerRef};
use crate::config::config::AppConfig;

use super::kafka_admin::ensure_auth_events_topic;
use super::kafka_client::base_client_config;

pub async fn spawn_auth_event_consumer_if_enabled(
    config: &AppConfig,
    auth_event_inbound_handler: AuthEventInboundHandlerRef,
) {
    if !config.kafka_enabled {
        return;
    }

    if let Err(e) = ensure_auth_events_topic(config).await {
        tracing::error!(
            error = %e,
            "failed to ensure kafka topic for auth events; consumer not started"
        );
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

    let auth_event_inbound_handler = Arc::clone(&auth_event_inbound_handler);
    tokio::spawn(async move {
        tracing::info!(
            service = "auth-service",
            topic = %topic,
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
                    match serde_json::from_str::<AuthEvent>(&body) {
                        Ok(event) => {
                            if let Err(e) = auth_event_inbound_handler.handle(event).await {
                                tracing::warn!(error = %e, "auth_event_inbound_handler error");
                            }
                        }
                        Err(e) => tracing::warn!(
                            error = %e,
                            topic = msg.topic(),
                            partition = msg.partition(),
                            offset = msg.offset(),
                            "invalid auth event payload"
                        ),
                    }
                }
                Err(e) => tracing::warn!(error = %e, "kafka stream error"),
            }
        }
        tracing::warn!("kafka consumer stream ended");
    });
}
