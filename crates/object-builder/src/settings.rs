use communication_utils::{
    consumer::{CommonConsumer, CommonConsumerConfig},
    publisher::CommonPublisher,
};
use derive_more::Display;
use serde::Deserialize;
use settings_utils::*;
use utils::notification::NotificationSettings;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub input_port: u16,
    pub chunk_capacity: usize,

    pub kafka: Option<ConsumerKafkaSettings>,
    pub amqp: Option<AmqpSettings>,

    pub services: ServicesSettings,

    pub monitoring: MonitoringSettings,

    #[serde(default)]
    pub log: LogSettings,

    #[serde(default)]
    pub notifications: NotificationSettings,
}

impl Settings {
    pub async fn consumer(&self) -> anyhow::Result<CommonConsumer> {
        match (&self.kafka, &self.amqp, &self.communication_method) {
            (Some(kafka), _, CommunicationMethod::Kafka) => {
                Ok(CommonConsumer::new(CommonConsumerConfig::Kafka {
                    brokers: &kafka.brokers,
                    group_id: &kafka.group_id,
                    topic: &kafka.ingest_topic,
                })
                .await?)
            }
            (_, Some(amqp), CommunicationMethod::Amqp) => {
                Ok(CommonConsumer::new(CommonConsumerConfig::Amqp {
                    connection_string: &amqp.exchange_url,
                    consumer_tag: &amqp.tag,
                    queue_name: &amqp.ingest_queue,
                    options: amqp.consume_options,
                })
                .await?)
            }
            _ => anyhow::bail!("Unsupported consumer specification"),
        }
    }

    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        publisher(
            self.kafka.as_ref().map(|kafka| kafka.brokers.as_str()),
            self.amqp.as_ref().map(|amqp| amqp.exchange_url.as_str()),
            None,
        )
        .await
    }
}

#[derive(Debug, Deserialize)]
pub struct ServicesSettings {
    pub schema_registry_url: String,
    pub edge_registry_url: String,
}

#[derive(Debug, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMethod {
    Kafka,
    Amqp,
    #[serde(other)]
    Other,
}
