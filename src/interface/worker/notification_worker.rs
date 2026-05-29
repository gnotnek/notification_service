use crate::{
    application::{
        command::send_notification::NotificationMessage,
        port::{
            notification_repository::NotificationRepository,
            notification_sender::NotificationSender,
        },
    },
    domain::notification_channel::NotificationChannel,
    infrastructure::{
        provider::{
            email_sender::ResendEmailSender, in_app_sender::MockInAppSender,
            push_sender::MockPushSender, whatsapp_sender::MockWhatsappSender,
        },
        rabbitmq::consumer::RabbitMqConsumer,
    },
    shared::error::AppResult,
};
use futures_lite::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions},
};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct NotificationWorker {
    repository: Arc<dyn NotificationRepository>,
    email_sender: Arc<dyn NotificationSender>,
    whatsapp_sender: Arc<dyn NotificationSender>,
    push_sender: Arc<dyn NotificationSender>,
    in_app_sender: Arc<dyn NotificationSender>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerProcessOutcome {
    Sent,
    MissingNotification,
    Failed { requeue: bool },
}

impl NotificationWorker {
    pub fn new(
        repository: Arc<dyn NotificationRepository>,
        resend_api_key: &str,
        resend_from_email: &str,
    ) -> AppResult<Self> {
        Ok(Self {
            repository,
            email_sender: Arc::new(ResendEmailSender::new(resend_api_key, resend_from_email)?),
            whatsapp_sender: Arc::new(MockWhatsappSender),
            push_sender: Arc::new(MockPushSender),
            in_app_sender: Arc::new(MockInAppSender),
        })
    }

    pub fn with_senders(
        repository: Arc<dyn NotificationRepository>,
        email_sender: Arc<dyn NotificationSender>,
        whatsapp_sender: Arc<dyn NotificationSender>,
        push_sender: Arc<dyn NotificationSender>,
        in_app_sender: Arc<dyn NotificationSender>,
    ) -> Self {
        Self {
            repository,
            email_sender,
            whatsapp_sender,
            push_sender,
            in_app_sender,
        }
    }

    pub async fn run_all(self: Arc<Self>, rabbitmq_url: String) -> AppResult<()> {
        let consumer = RabbitMqConsumer::connect(&rabbitmq_url, 20).await?;

        for channel in NotificationChannel::all() {
            let worker = Arc::clone(&self);
            let consumer = consumer.clone();
            tokio::spawn(async move {
                if let Err(err) = worker.consume_channel(consumer, channel).await {
                    error!(error = %err, channel = %channel, "notification worker stopped");
                }
            });
        }

        Ok(())
    }

    async fn consume_channel(
        self: Arc<Self>,
        consumer: RabbitMqConsumer,
        channel: NotificationChannel,
    ) -> AppResult<()> {
        let mut deliveries = consumer
            .consume(
                channel.queue_name(),
                &format!("notification-{}-worker", channel.as_str()),
            )
            .await?;

        info!(channel = %channel, queue = channel.queue_name(), "notification worker started");

        while let Some(delivery) = deliveries.next().await {
            match delivery {
                Ok(delivery) => self.handle_delivery(delivery).await?,
                Err(err) => error!(error = %err, channel = %channel, "rabbitmq delivery error"),
            }
        }

        Ok(())
    }

    async fn handle_delivery(&self, delivery: Delivery) -> AppResult<()> {
        let message: NotificationMessage = match serde_json::from_slice(&delivery.data) {
            Ok(message) => message,
            Err(err) => {
                warn!(error = %err, "invalid notification message; acknowledging");
                delivery.ack(BasicAckOptions::default()).await?;
                return Ok(());
            }
        };

        match self.process_message(message, delivery.redelivered).await? {
            WorkerProcessOutcome::Sent => {
                delivery.ack(BasicAckOptions::default()).await?;
            }
            WorkerProcessOutcome::MissingNotification => {
                delivery.ack(BasicAckOptions::default()).await?;
            }
            WorkerProcessOutcome::Failed { requeue } => {
                delivery
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue,
                    })
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn process_message(
        &self,
        message: NotificationMessage,
        redelivered: bool,
    ) -> AppResult<WorkerProcessOutcome> {
        let Some(notification) = self.repository.find_by_id(message.notification_id).await? else {
            warn!(
                notification_id = %message.notification_id,
                "notification not found; acknowledging"
            );
            return Ok(WorkerProcessOutcome::MissingNotification);
        };

        let sender = self.sender_for(message.channel)?;

        match sender.send(&notification).await {
            Ok(response) => {
                self.repository
                    .record_attempt(
                        &notification,
                        "success",
                        Some(json!({
                            "provider": response.provider,
                            "response": response.response,
                        })),
                        None,
                    )
                    .await?;
                self.repository.mark_sent(notification.id).await?;
                info!(notification_id = %notification.id, "notification processed");
                Ok(WorkerProcessOutcome::Sent)
            }
            Err(err) => {
                self.repository
                    .record_attempt(&notification, "failed", None, Some(err.to_string()))
                    .await?;
                let requeue = !redelivered;
                if !requeue {
                    self.repository.mark_failed(notification.id).await?;
                }
                warn!(
                    notification_id = %notification.id,
                    error = %err,
                    requeue,
                    "notification processing failed; message nacked"
                );
                Ok(WorkerProcessOutcome::Failed { requeue })
            }
        }
    }

    fn sender_for(&self, channel: NotificationChannel) -> AppResult<Arc<dyn NotificationSender>> {
        match channel {
            NotificationChannel::Email => Ok(Arc::clone(&self.email_sender)),
            NotificationChannel::Whatsapp => Ok(Arc::clone(&self.whatsapp_sender)),
            NotificationChannel::Push => Ok(Arc::clone(&self.push_sender)),
            NotificationChannel::InApp => Ok(Arc::clone(&self.in_app_sender)),
        }
    }
}
