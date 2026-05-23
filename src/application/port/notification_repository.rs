use crate::{
    application::command::create_notification::CreateNotificationCommand,
    domain::notification::Notification, shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create_with_outbox(&self, command: CreateNotificationCommand) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Notification>>;
    async fn mark_sent(&self, id: Uuid) -> AppResult<()>;
    async fn mark_failed(&self, id: Uuid) -> AppResult<()>;
    async fn record_attempt(
        &self,
        notification: &Notification,
        status: &str,
        provider_response: Option<Value>,
        error_message: Option<String>,
    ) -> AppResult<()>;
}
