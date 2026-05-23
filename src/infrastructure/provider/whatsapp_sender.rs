use crate::{
    application::port::notification_sender::{NotificationSender, ProviderResponse},
    domain::notification::Notification,
    shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MockWhatsappSender;

#[async_trait]
impl NotificationSender for MockWhatsappSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        info!(
            notification_id = %notification.id,
            recipient = %notification.recipient,
            "mock whatsapp sent"
        );

        Ok(ProviderResponse {
            provider: "mock_whatsapp",
            response: json!({
                "message_id": format!("whatsapp-{}", notification.id),
                "status": "accepted"
            }),
        })
    }
}
