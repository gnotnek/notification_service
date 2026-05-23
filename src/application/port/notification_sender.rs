use crate::{domain::notification::Notification, shared::error::AppResult};
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub provider: &'static str,
    pub response: Value,
}

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse>;
}
