use crate::{
    application::port::notification_sender::{NotificationSender, ProviderResponse},
    domain::notification::Notification,
    shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MockEmailSender;

#[async_trait]
impl NotificationSender for MockEmailSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        info!(
            notification_id = %notification.id,
            recipient = %notification.recipient,
            "mock email sent"
        );

        Ok(ProviderResponse {
            provider: "mock_email",
            response: json!({
                "message_id": format!("email-{}", notification.id),
                "status": "accepted"
            }),
        })
    }
}
