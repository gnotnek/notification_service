mod common;

use common::sample_notification;
use notification_service::{
    application::port::notification_sender::NotificationSender,
    domain::notification_channel::NotificationChannel,
    infrastructure::provider::{
        email_sender::MockEmailSender, in_app_sender::MockInAppSender, push_sender::MockPushSender,
        whatsapp_sender::MockWhatsappSender,
    },
};

#[tokio::test]
async fn mock_providers_return_provider_specific_responses() {
    let notification = sample_notification(NotificationChannel::Email);

    let email = MockEmailSender.send(&notification).await.unwrap();
    assert_eq!(email.provider, "mock_email");
    assert_eq!(email.response["status"], "accepted");

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
