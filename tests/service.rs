mod common;

use common::{MockNotificationRepository, create_command};
use notification_service::{
    application::service::notification_service::NotificationService,
    domain::notification_channel::NotificationChannel, shared::error::AppError,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn notification_service_creates_valid_notification() {
    let repository = Arc::new(MockNotificationRepository::default());
    let expected_id = Uuid::new_v4();
    *repository.created_id.lock().unwrap() = Some(expected_id);
    let service = NotificationService::new(repository.clone());

    let result = service
        .create(create_command(NotificationChannel::Email))
        .await
        .unwrap();

    assert_eq!(result.notification_id, expected_id);
    let created = repository.created.lock().unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].channel, NotificationChannel::Email);
}

#[tokio::test]
async fn notification_service_rejects_empty_recipient() {
    let repository = Arc::new(MockNotificationRepository::default());
    let service = NotificationService::new(repository);
    let mut command = create_command(NotificationChannel::Email);
    command.recipient = "   ".to_string();

    let error = service.create(command).await.unwrap_err();

    assert!(matches!(error, AppError::Validation(message) if message == "recipient is required"));
}

#[tokio::test]
async fn notification_service_rejects_empty_body() {
    let repository = Arc::new(MockNotificationRepository::default());
    let service = NotificationService::new(repository);
    let mut command = create_command(NotificationChannel::Email);
    command.body = "\n\t".to_string();

    let error = service.create(command).await.unwrap_err();

    assert!(matches!(error, AppError::Validation(message) if message == "body is required"));
}
