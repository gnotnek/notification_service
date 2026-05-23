use crate::{
    application::{
        command::send_notification::NotificationMessage,
        port::{
            message_broker::MessageBroker,
            outbox_repository::{OutboxEvent, OutboxRepository},
        },
    },
    shared::error::AppResult,
};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct OutboxPublisher {
    outbox_repository: Arc<dyn OutboxRepository>,
    publisher: Arc<dyn MessageBroker>,
    batch_size: i64,
}

impl OutboxPublisher {
    pub fn new(
        outbox_repository: Arc<dyn OutboxRepository>,
        publisher: Arc<dyn MessageBroker>,
        batch_size: i64,
    ) -> Self {
        Self {
            outbox_repository,
            publisher,
            batch_size,
        }
    }

    pub async fn publish_once(&self) -> AppResult<usize> {
        let events = self
            .outbox_repository
            .claim_pending(self.batch_size)
            .await?;
        let total = events.len();

        for event in events {
            if let Err(err) = self.publish_event(&event).await {
                warn!(
                    error = %err,
                    outbox_id = %event.id,
                    "failed to publish outbox event"
                );
                self.outbox_repository
                    .mark_failed_for_retry(event.id)
                    .await?;
            }
        }

        Ok(total)
    }

    pub async fn run(self: Arc<Self>, interval: Duration) {
        info!("outbox publisher started");

        loop {
            match self.publish_once().await {
                Ok(count) if count > 0 => info!(count, "published outbox batch"),
                Ok(_) => {}
                Err(err) => error!(error = %err, "outbox publisher failed"),
            }

            tokio::time::sleep(interval).await;
        }
    }

    async fn publish_event(&self, event: &OutboxEvent) -> AppResult<()> {
        let message: NotificationMessage = serde_json::from_value(event.payload.clone())?;
        info!(
            outbox_id = %event.id,
            notification_id = %event.notification_id,
            event_type = %event.event_type,
            retry_count = event.retry_count,
            routing_key = message.channel.routing_key(),
            "publishing outbox event"
        );
        self.publisher.publish(message.channel, &message).await?;
        self.outbox_repository.mark_published(event.id).await
    }
}
