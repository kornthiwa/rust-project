use std::time::Duration;

use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::types::RDKafkaErrorCode;

use crate::config::config::AppConfig;

use super::kafka_client::base_client_config;

pub async fn ensure_message_events_topic(config: &AppConfig) -> Result<(), KafkaError> {
    let admin_cfg = base_client_config(config, "admin");
    let admin: AdminClient<DefaultClientContext> = AdminClient::from_config(&admin_cfg)?;

    let topic_name = config.kafka_topic_message_events.as_str();
    let new_topic = NewTopic::new(topic_name, 1, TopicReplication::Fixed(1));
    let opts = AdminOptions::new().request_timeout(Some(Duration::from_secs(30)));

    let results = admin.create_topics([&new_topic], &opts).await?;

    for result in results {
        match result {
            Ok(_) => tracing::info!(topic = topic_name, created = true, "kafka topic ensured"),
            Err((_name, RDKafkaErrorCode::TopicAlreadyExists)) => {
                tracing::info!(topic = topic_name, created = false, "kafka topic ensured");
            }
            Err((name, code)) => {
                tracing::error!(topic = %name, ?code, "kafka admin create topic failed");
                return Err(KafkaError::AdminOpCreation(format!(
                    "create topic {name}: {code:?}"
                )));
            }
        }
    }

    Ok(())
}

pub async fn ensure_auth_events_topic(config: &AppConfig) -> Result<(), KafkaError> {
    let admin_cfg = base_client_config(config, "admin");
    let admin: AdminClient<DefaultClientContext> = AdminClient::from_config(&admin_cfg)?;

    let topic_name = config.kafka_topic_auth_events.as_str();
    let new_topic = NewTopic::new(topic_name, 1, TopicReplication::Fixed(1));
    let opts = AdminOptions::new().request_timeout(Some(Duration::from_secs(30)));

    let results = admin.create_topics([&new_topic], &opts).await?;

    for result in results {
        match result {
            Ok(_) => tracing::info!(topic = topic_name, created = true, "kafka topic ensured"),
            Err((_name, RDKafkaErrorCode::TopicAlreadyExists)) => {
                tracing::info!(topic = topic_name, created = false, "kafka topic ensured");
            }
            Err((name, code)) => {
                tracing::error!(topic = %name, ?code, "kafka admin create topic failed");
                return Err(KafkaError::AdminOpCreation(format!(
                    "create topic {name}: {code:?}"
                )));
            }
        }
    }

    Ok(())
}
