mod common;

use common::{MockBroker, MockOutboxRepository, outbox_event};
use notification_service::{
    application::{
        port::outbox_repository::OutboxEvent, service::outbox_publisher::OutboxPublisher,
    },
    domain::notification_channel::NotificationChannel,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn outbox_publisher_publishes_and_marks_event() {
    let event = outbox_event(NotificationChannel::Push);
    let repository = Arc::new(MockOutboxRepository::new(vec![event.clone()]));
    let broker = Arc::new(MockBroker::default());
    let publisher = OutboxPublisher::new(repository.clone(), broker.clone(), 10);

    let count = publisher.publish_once().await.unwrap();

    assert_eq!(count, 1);
    assert_eq!(*repository.batch_sizes.lock().unwrap(), vec![10]);
    assert_eq!(*repository.published.lock().unwrap(), vec![event.id]);
    assert!(repository.failed_for_retry.lock().unwrap().is_empty());
    let published = broker.published.lock().unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].0, NotificationChannel::Push);
}

#[tokio::test]
async fn outbox_publisher_marks_invalid_payload_for_retry() {
    let event = OutboxEvent {
        id: Uuid::new_v4(),
        notification_id: Uuid::new_v4(),
        event_type: "notification.created".to_string(),
        payload: json!({ "unexpected": true }),
        retry_count: 0,
    };
    let repository = Arc::new(MockOutboxRepository::new(vec![event.clone()]));
    let broker = Arc::new(MockBroker::default());
    let publisher = OutboxPublisher::new(repository.clone(), broker.clone(), 10);

    let count = publisher.publish_once().await.unwrap();

    assert_eq!(count, 1);
    assert!(repository.published.lock().unwrap().is_empty());
    assert_eq!(*repository.failed_for_retry.lock().unwrap(), vec![event.id]);
    assert!(broker.published.lock().unwrap().is_empty());
}

#[tokio::test]
async fn outbox_publisher_marks_broker_failure_for_retry() {
    let event = outbox_event(NotificationChannel::Whatsapp);
    let repository = Arc::new(MockOutboxRepository::new(vec![event.clone()]));
    let broker = Arc::new(MockBroker {
        fail: true,
        ..Default::default()
    });
    let publisher = OutboxPublisher::new(repository.clone(), broker, 10);

    let count = publisher.publish_once().await.unwrap();

    assert_eq!(count, 1);
    assert!(repository.published.lock().unwrap().is_empty());
    assert_eq!(*repository.failed_for_retry.lock().unwrap(), vec![event.id]);
}
