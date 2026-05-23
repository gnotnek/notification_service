use crate::{
    application::port::notification_sender::{NotificationSender, ProviderResponse},
    domain::notification::Notification,
    shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MockInAppSender;

#[async_trait]
impl NotificationSender for MockInAppSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        info!(
            notification_id = %notification.id,
            recipient = %notification.recipient,
            "mock in-app notification stored"
        );

        Ok(ProviderResponse {
            provider: "mock_in_app",
            response: json!({
                "message_id": format!("in-app-{}", notification.id),
                "status": "stored"
            }),
        })
    }
}
