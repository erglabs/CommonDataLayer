use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use cdl_dto::ingestion::{BorrowedInsertMessage, OwnedInsertMessage};
use metrics_utils::*;
use std::sync::Arc;
use tracing::trace;
use utils::notification::NotificationPublisher;

pub mod config;
pub mod resolution;

pub struct MessageRouter<P: OutputPlugin> {
    notification_sender: Arc<NotificationPublisher<OwnedInsertMessage>>,
    output_plugin: Arc<P>,
}

impl<P: OutputPlugin> Clone for MessageRouter<P> {
    fn clone(&self) -> Self {
        MessageRouter {
            notification_sender: self.notification_sender.clone(),
            output_plugin: self.output_plugin.clone(),
        }
    }
}

impl<P: OutputPlugin> MessageRouter<P> {
    pub fn new(report_sender: NotificationPublisher<OwnedInsertMessage>, output_plugin: P) -> Self {
        Self {
            notification_sender: Arc::new(report_sender),
            output_plugin: Arc::new(output_plugin),
        }
    }

    #[tracing::instrument(skip(self, msg))]
    pub async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> anyhow::Result<()> {
        let instance = Arc::clone(&self.notification_sender).and_message_body(&msg);

        let status = self.output_plugin.handle_message(msg).await;

        trace!("Finished processing a message with resolution `{}`", status);

        match status {
            Resolution::StorageLayerFailure { .. } => {
                counter!("cdl.command-service.post-process.storage-failure", 1);
            }
            Resolution::CommandServiceFailure => {
                counter!("cdl.command-service.post-process.command-failure", 1);
            }
            Resolution::UserFailure { .. } => {
                counter!("cdl.command-service.post-process.user-failure", 1);
            }
            Resolution::Success => {
                counter!("cdl.command-service.post-process.success", 1);
            }
        }

        instance.notify(&status.to_string()).await?;

        Ok(())
    }
}
