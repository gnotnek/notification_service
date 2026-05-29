use crate::{
    application::port::notification_sender::{NotificationSender, ProviderResponse},
    domain::notification::Notification,
    shared::error::{AppError, AppResult},
};
use async_trait::async_trait;
use resend_rs::{
    Resend,
    types::{CreateEmailBaseOptions, Tag},
};
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ResendEmailSender {
    client: Resend,
    from_email: String,
}

impl ResendEmailSender {
    pub fn new(api_key: &str, from_email: &str) -> AppResult<Self> {
        if api_key.trim().is_empty() {
            return Err(AppError::Config("RESEND_API_KEY is required".to_string()));
        }

        if from_email.trim().is_empty() {
            return Err(AppError::Config(
                "RESEND_FROM_EMAIL is required".to_string(),
            ));
        }

        Ok(Self {
            client: Resend::new(api_key.trim()),
            from_email: from_email.trim().to_string(),
        })
    }

    pub fn email_options_for(&self, notification: &Notification) -> CreateEmailBaseOptions {
        build_email_options(&self.from_email, notification)
    }
}

#[async_trait]
impl NotificationSender for ResendEmailSender {
    async fn send(&self, notification: &Notification) -> AppResult<ProviderResponse> {
        let idempotency_key = format!("notification-{}", notification.id);
        let email = self
            .email_options_for(notification)
            .with_idempotency_key(&idempotency_key);
        let response = self
            .client
            .emails
            .send(email)
            .await
            .map_err(|err| AppError::Provider(format!("resend send email failed: {err}")))?;

        info!(
            notification_id = %notification.id,
            resend_email_id = %response.id,
            recipient = %notification.recipient,
            "email sent with resend"
        );

        Ok(ProviderResponse {
            provider: "resend",
            response: json!({
                "email_id": response.id.to_string(),
                "status": "sent"
            }),
        })
    }
}

pub fn build_email_options(
    from_email: &str,
    notification: &Notification,
) -> CreateEmailBaseOptions {
    let subject = notification.subject.as_deref().unwrap_or("Notification");
    let mut options =
        CreateEmailBaseOptions::new(from_email, [notification.recipient.as_str()], subject)
            .with_text(&notification.body)
            .with_tag(Tag::new("notification_id", &notification.id.to_string()))
            .with_tag(Tag::new("channel", notification.channel.as_str()));

    if let Some(html) = notification
        .payload
        .as_ref()
        .and_then(|payload| payload.get("html"))
        .and_then(|html| html.as_str())
    {
        options = options.with_html(html);
    }

    if let Some(scheduled_at) = notification.scheduled_at {
        options = options.with_scheduled_at(&scheduled_at.to_rfc3339());
    }

    options
}
