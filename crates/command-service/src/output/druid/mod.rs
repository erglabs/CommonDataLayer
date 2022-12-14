use std::time::Duration;

use anyhow::Context;
use cdl_dto::ingestion::BorrowedInsertMessage;
use futures::stream::{self, StreamExt};
use metrics_utils::{self as metrics, counter};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use settings_utils::apps::command_service::CommandServiceDruidSettings;
use tracing::error;
use uuid::Uuid;

use crate::{communication::resolution::Resolution, output::OutputPlugin};

#[derive(Deserialize)]
struct TimeseriesInputMessage {
    fields: Value,
    ts: u64,
}

#[derive(Serialize)]
struct DruidOutputMessage {
    fields: Value,
    ts: u64,
    object_id: Uuid,
    schema_id: Uuid,
}

pub struct DruidOutputPlugin {
    producer: FutureProducer,
    topic: String,
}

impl DruidOutputPlugin {
    pub async fn new(settings: CommandServiceDruidSettings, brokers: &str) -> anyhow::Result<Self> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .context("Producer creation")?,
            topic: settings.topic,
        })
    }
}

#[async_trait::async_trait]
impl OutputPlugin for DruidOutputPlugin {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let key = msg.object_id.to_string();
        let payloads = match deserialize_payloads(&msg) {
            Ok(result) => result,
            Err(err) => return err,
        };
        let messages: Vec<FutureRecord<_, _>> = payloads
            .iter()
            .map(|payload| FutureRecord {
                topic: &self.topic,
                partition: None,
                payload: Some(payload),
                key: Some(&key),
                timestamp: Some(msg.timestamp),
                headers: None,
            })
            .collect();
        stream::iter(messages)
            .then(|message| async {
                match self.producer.send(message, Duration::from_secs(0)).await {
                    Err((err, _)) => Err(Resolution::StorageLayerFailure {
                        description: err.to_string(),
                    }),
                    Ok(_) => {
                        counter!("cdl.command-service.store.druid", 1);
                        Ok(Resolution::Success)
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;
        Resolution::Success
    }

    fn name(&self) -> &'static str {
        "Druid timeseries"
    }
}

fn deserialize_payloads(msg: &BorrowedInsertMessage<'_>) -> Result<Vec<Vec<u8>>, Resolution> {
    let result: Result<Vec<TimeseriesInputMessage>, serde_json::Error> =
        serde_json::from_str(msg.data.get());
    match result {
        Ok(values) => Ok(values
            .into_iter()
            .map(|payload: TimeseriesInputMessage| {
                serde_json::to_vec(&DruidOutputMessage {
                    fields: payload.fields,
                    ts: payload.ts,
                    object_id: msg.object_id,
                    schema_id: msg.schema_id,
                })
                .unwrap()
            })
            .collect()),
        Err(err) => {
            let context = msg.data.to_string();
            error!(
                "Failed to read payload, cause `{}`, context `{}`",
                err, context
            );
            Err(Resolution::UserFailure {
                description: err.to_string(),
                context,
            })
        }
    }
}
