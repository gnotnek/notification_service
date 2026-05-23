#![allow(dead_code)]

use async_trait::async_trait;
use chrono::Utc;
use notification_service::{
    application::{
        command::{
            create_notification::CreateNotificationCommand, send_notification::NotificationMessage,
        },
        port::{
            message_broker::MessageBroker,
            notification_repository::NotificationRepository,
            notification_sender::{NotificationSender, ProviderResponse},
            outbox_repository::{OutboxEvent, OutboxRepository},
        },
    },
    domain::{
        notification::Notification, notification_channel::NotificationChannel,
        notification_status::NotificationStatus,
    },
    interface::worker::notification_worker::NotificationWorker,
    shared::error::{AppError, AppResult},
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Default)]
pub struct MockNotificationRepository {
    pub created: Mutex<Vec<CreateNotificationCommand>>,
    pub find_result: Mutex<Option<Notification>>,
    pub attempts: Mutex<Vec<AttemptRecord>>,
    pub marked_sent: Mutex<Vec<Uuid>>,
    pub marked_failed: Mutex<Vec<Uuid>>,
    pub created_id: Mutex<Option<Uuid>>,
}

#[derive(Debug)]
pub struct AttemptRecord {
    pub notification_id: Uuid,
    pub status: String,
    pub provider_response: Option<Value>,
    pub error_message: Option<String>,
}

#[async_trait]
impl NotificationRepository for MockNotificationRepository {
    async fn create_with_outbox(&self, command: CreateNotificationCommand) -> AppResult<Uuid> {
        self.created.lock().unwrap().push(command);
        let id = self.created_id.lock().unwrap().unwrap_or_else(Uuid::new_v4);
        Ok(id)
    }

    async fn find_by_id(&self, _id: Uuid) -> AppResult<Option<Notification>> {
        Ok(self.find_result.lock().unwrap().clone())
    }

    async fn mark_sent(&self, id: Uuid) -> AppResult<()> {
        self.marked_sent.lock().unwrap().push(id);
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid) -> AppResult<()> {
        self.marked_failed.lock().unwrap().push(id);
        Ok(())
    }

    async fn record_attempt(
        &self,
        notification: &Notification,
        status: &str,
        provider_response: Option<Value>,
        error_message: Option<String>,
    ) -> AppResult<()> {
        self.attempts.lock().unwrap().push(AttemptRecord {
            notification_id: notification.id,
            status: status.to_string(),
            provider_response,
            error_message,
        });
        Ok(())
    }
}

pub struct MockOutboxRepository {
    pub events: Mutex<VecDeque<OutboxEvent>>,
    pub batch_sizes: Mutex<Vec<i64>>,
    pub published: Mutex<Vec<Uuid>>,
    pub failed_for_retry: Mutex<Vec<Uuid>>,
}

impl MockOutboxRepository {
    pub fn new(events: Vec<OutboxEvent>) -> Self {
        Self {
            events: Mutex::new(events.into()),
            batch_sizes: Mutex::new(Vec::new()),
            published: Mutex::new(Vec::new()),
            failed_for_retry: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl OutboxRepository for MockOutboxRepository {
    async fn claim_pending(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>> {
        self.batch_sizes.lock().unwrap().push(batch_size);
        let mut events = self.events.lock().unwrap();
        let mut claimed = Vec::new();

        for _ in 0..batch_size {
            let Some(event) = events.pop_front() else {
                break;
            };
            claimed.push(event);
        }

        Ok(claimed)
    }

    async fn mark_published(&self, id: Uuid) -> AppResult<()> {
        self.published.lock().unwrap().push(id);
        Ok(())
    }

    async fn mark_failed_for_retry(&self, id: Uuid) -> AppResult<()> {
        self.failed_for_retry.lock().unwrap().push(id);
        Ok(())
    }
}

#[derive(Default)]
pub struct MockBroker {
    pub fail: bool,
    pub published: Mutex<Vec<(NotificationChannel, NotificationMessage)>>,
}

#[async_trait]
impl MessageBroker for MockBroker {
    async fn publish(
        &self,
        channel: NotificationChannel,
        message: &NotificationMessage,
    ) -> AppResult<()> {
        if self.fail {
            return Err(AppError::Internal("publish failed".to_string()));
        }

        self.published
            .lock()
            .unwrap()
            .push((channel, message.clone()));
        Ok(())
    }
}

pub struct MockSender {
    pub provider: &'static str,
    pub fail: bool,
}

#[async_trait]
impl NotificationSender for MockSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        if self.fail {
            return Err(AppError::Internal(format!(
                "{} failed for {}",
                self.provider, notification.id
            )));
        }

        Ok(ProviderResponse {
            provider: self.provider,
            response: json!({ "status": "accepted" }),
        })
    }
}

pub fn create_command(channel: NotificationChannel) -> CreateNotificationCommand {
    CreateNotificationCommand {
        recipient: "user@example.com".to_string(),
        channel,
        subject: Some("Welcome".to_string()),
        body: "Your account is ready".to_string(),
        payload: Some(json!({ "template": "welcome" })),
        scheduled_at: None,
    }
}

pub fn sample_notification(channel: NotificationChannel) -> Notification {
    let now = Utc::now();

    Notification {
        id: Uuid::new_v4(),
        recipient: "user@example.com".to_string(),
        channel,
        subject: Some("Welcome".to_string()),
        body: "Your account is ready".to_string(),
        payload: Some(json!({ "template": "welcome" })),
        status: NotificationStatus::Pending,
        scheduled_at: None,
        sent_at: None,
        failed_at: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn outbox_event(channel: NotificationChannel) -> OutboxEvent {
    let notification_id = Uuid::new_v4();

    OutboxEvent {
        id: Uuid::new_v4(),
        notification_id,
        event_type: "notification.created".to_string(),
        payload: json!(NotificationMessage {
            notification_id,
            channel
        }),
        retry_count: 0,
    }
}

pub fn worker_with(
    repository: Arc<MockNotificationRepository>,
    sender: Arc<dyn NotificationSender>,
) -> NotificationWorker {
    NotificationWorker::with_senders(
        repository,
        Arc::clone(&sender),
        Arc::clone(&sender),
        Arc::clone(&sender),
        sender,
    )
}
