mod common;

use common::{MockNotificationRepository, MockSender, sample_notification, worker_with};
use notification_service::{
    application::command::send_notification::NotificationMessage,
    domain::notification_channel::NotificationChannel,
    interface::worker::notification_worker::WorkerProcessOutcome,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn worker_processes_successful_message() {
    let repository = Arc::new(MockNotificationRepository::default());
    let notification = sample_notification(NotificationChannel::Email);
    *repository.find_result.lock().unwrap() = Some(notification.clone());
    let worker = worker_with(
        repository.clone(),
        Arc::new(MockSender {
            provider: "test",
            fail: false,
        }),
    );

    let outcome = worker
        .process_message(
            NotificationMessage {
                notification_id: notification.id,
                channel: NotificationChannel::Email,
            },
            false,
        )
        .await
        .unwrap();

    assert_eq!(outcome, WorkerProcessOutcome::Sent);
    assert_eq!(
        *repository.marked_sent.lock().unwrap(),
        vec![notification.id]
    );
    assert!(repository.marked_failed.lock().unwrap().is_empty());
    let attempts = repository.attempts.lock().unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].notification_id, notification.id);
    assert_eq!(attempts[0].status, "success");
    assert!(attempts[0].provider_response.is_some());
}

#[tokio::test]
async fn worker_acknowledges_missing_notification_without_attempt() {
    let repository = Arc::new(MockNotificationRepository::default());
    let worker = worker_with(
        repository.clone(),
        Arc::new(MockSender {
            provider: "test",
            fail: false,
        }),
    );

    let outcome = worker
        .process_message(
            NotificationMessage {
                notification_id: Uuid::new_v4(),
                channel: NotificationChannel::Email,
            },
            false,
        )
        .await
        .unwrap();

    assert_eq!(outcome, WorkerProcessOutcome::MissingNotification);
    assert!(repository.attempts.lock().unwrap().is_empty());
    assert!(repository.marked_sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn worker_requeues_first_provider_failure() {
    let repository = Arc::new(MockNotificationRepository::default());
    let notification = sample_notification(NotificationChannel::Email);
    *repository.find_result.lock().unwrap() = Some(notification.clone());
    let worker = worker_with(
        repository.clone(),
        Arc::new(MockSender {
            provider: "test",
            fail: true,
        }),
    );

    let outcome = worker
        .process_message(
            NotificationMessage {
                notification_id: notification.id,
                channel: NotificationChannel::Email,
            },
            false,
        )
        .await
        .unwrap();

    assert_eq!(outcome, WorkerProcessOutcome::Failed { requeue: true });
    assert!(repository.marked_failed.lock().unwrap().is_empty());
    let attempts = repository.attempts.lock().unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, "failed");
    assert!(attempts[0].error_message.is_some());
}

#[tokio::test]
async fn worker_marks_failed_after_redelivered_provider_failure() {
    let repository = Arc::new(MockNotificationRepository::default());
    let notification = sample_notification(NotificationChannel::Push);
    *repository.find_result.lock().unwrap() = Some(notification.clone());
    let worker = worker_with(
        repository.clone(),
        Arc::new(MockSender {
            provider: "test",
            fail: true,
        }),
    );

    let outcome = worker
        .process_message(
            NotificationMessage {
                notification_id: notification.id,
                channel: NotificationChannel::Push,
            },
            true,
        )
        .await
        .unwrap();

    assert_eq!(outcome, WorkerProcessOutcome::Failed { requeue: false });
    assert_eq!(
        *repository.marked_failed.lock().unwrap(),
        vec![notification.id]
    );
}
