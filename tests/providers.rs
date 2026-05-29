mod common;

use common::sample_notification;
use notification_service::{
    application::port::notification_sender::NotificationSender,
    domain::notification_channel::NotificationChannel,
    infrastructure::provider::{
        email_sender::{ResendEmailSender, build_email_options},
        in_app_sender::MockInAppSender,
        push_sender::MockPushSender,
        whatsapp_sender::MockWhatsappSender,
    },
    shared::error::AppError,
};
use serde_json::{Value, json};

#[test]
fn resend_email_sender_requires_api_key() {
    let error = ResendEmailSender::new("", "Notifications <notifications@example.com>")
        .expect_err("empty api key should fail");

    assert!(matches!(error, AppError::Config(message) if message == "RESEND_API_KEY is required"));
}

#[test]
fn resend_email_sender_requires_from_email() {
    let error = ResendEmailSender::new("re_test", " ").expect_err("empty from email should fail");

    assert!(
        matches!(error, AppError::Config(message) if message == "RESEND_FROM_EMAIL is required")
    );
}

#[test]
fn resend_email_options_include_notification_content() {
    let mut notification = sample_notification(NotificationChannel::Email);
    notification.payload = Some(json!({
        "html": "<p>Your account is ready</p>"
    }));

    let email = build_email_options("Notifications <notifications@example.com>", &notification);
    let value = serde_json::to_value(email).unwrap();

    assert_eq!(value["from"], "Notifications <notifications@example.com>");
    assert_eq!(value["to"], json!(["user@example.com"]));
    assert_eq!(value["subject"], "Welcome");
    assert_eq!(value["text"], "Your account is ready");
    assert_eq!(value["html"], "<p>Your account is ready</p>");
    assert_eq!(
        value["tags"],
        json!([
            {"name": "notification_id", "value": notification.id.to_string()},
            {"name": "channel", "value": "email"}
        ])
    );
}

#[test]
fn resend_email_options_use_default_subject() {
    let mut notification = sample_notification(NotificationChannel::Email);
    notification.subject = None;

    let value: Value = serde_json::to_value(build_email_options(
        "Notifications <notifications@example.com>",
        &notification,
    ))
    .unwrap();

    assert_eq!(value["subject"], "Notification");
}

#[tokio::test]
async fn mock_non_email_providers_return_provider_specific_responses() {
    let notification = sample_notification(NotificationChannel::Whatsapp);

    let whatsapp = MockWhatsappSender.send(&notification).await.unwrap();
    assert_eq!(whatsapp.provider, "mock_whatsapp");
    assert_eq!(whatsapp.response["status"], "accepted");

    let push = MockPushSender.send(&notification).await.unwrap();
    assert_eq!(push.provider, "mock_push");
    assert_eq!(push.response["status"], "accepted");

    let in_app = MockInAppSender.send(&notification).await.unwrap();
    assert_eq!(in_app.provider, "mock_in_app");
    assert_eq!(in_app.response["status"], "stored");
}
