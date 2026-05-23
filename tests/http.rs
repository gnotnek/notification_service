mod common;

use axum::{Json, extract::State, http::StatusCode};
use common::MockNotificationRepository;
use notification_service::{
    application::service::notification_service::NotificationService,
    domain::notification_channel::NotificationChannel,
    interface::http::{
        dto::CreateNotificationRequest,
        handler::create_notification,
        route::{HttpState, router},
    },
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[test]
fn create_request_deserializes_snake_case_channel() {
    let request: CreateNotificationRequest = serde_json::from_value(json!({
        "recipient": "user@example.com",
        "channel": "in_app",
        "body": "Hello",
        "payload": { "screen": "inbox" }
    }))
    .unwrap();

    assert_eq!(request.channel, NotificationChannel::InApp);
    assert_eq!(request.recipient, "user@example.com");
    assert_eq!(request.body, "Hello");
}

#[test]
fn router_builds_notification_route() {
    let repository = Arc::new(MockNotificationRepository::default());
    let state = HttpState {
        notification_service: Arc::new(NotificationService::new(repository)),
    };

    let _router = router(state);
}

#[tokio::test]
async fn http_handler_returns_created_notification_id() {
    let repository = Arc::new(MockNotificationRepository::default());
    let expected_id = Uuid::new_v4();
    *repository.created_id.lock().unwrap() = Some(expected_id);
    let state = HttpState {
        notification_service: Arc::new(NotificationService::new(repository)),
    };

    let (status, Json(response)) = create_notification(
        State(state),
        Json(CreateNotificationRequest {
            recipient: "user@example.com".to_string(),
            channel: NotificationChannel::Email,
            subject: None,
            body: "Hello".to_string(),
            payload: None,
            scheduled_at: None,
        }),
    )
    .await
    .unwrap();

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response.notification_id, expected_id);
}
