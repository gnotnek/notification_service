use crate::{
    application::port::notification_sender::{NotificationSender, ProviderResponse},
    domain::notification::Notification,
    shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MockPushSender;

#[async_trait]
impl NotificationSender for MockPushSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        info!(
            notification_id = %notification.id,
            recipient = %notification.recipient,
            "mock push notification sent"
        );

        Ok(ProviderResponse {
            provider: "mock_push",
            response: json!({
                "message_id": format!("push-{}", notification.id),
                "status": "accepted"
            }),
        })
    }
}
