use axum::{http::StatusCode, response::IntoResponse};
use notification_service::{
    domain::{notification_channel::NotificationChannel, notification_status::NotificationStatus},
    shared::error::AppError,
};
use std::str::FromStr;

#[test]
fn notification_channel_metadata_matches_queue_design() {
    let email = NotificationChannel::Email;
    assert_eq!(email.as_str(), "email");
    assert_eq!(email.to_string(), "email");
    assert_eq!(email.routing_key(), "notification.email");
    assert_eq!(email.queue_name(), "notification.email.queue");
    assert_eq!(email.dlq_name(), Some("notification.email.dlq"));
    assert_eq!(email.dlq_routing_key(), Some("notification.email.dlq"));

    let whatsapp = NotificationChannel::Whatsapp;
    assert_eq!(whatsapp.as_str(), "whatsapp");
    assert_eq!(whatsapp.to_string(), "whatsapp");
    assert_eq!(whatsapp.routing_key(), "notification.whatsapp");
    assert_eq!(whatsapp.queue_name(), "notification.whatsapp.queue");
    assert_eq!(whatsapp.dlq_name(), Some("notification.whatsapp.dlq"));

    let push = NotificationChannel::Push;
    assert_eq!(push.as_str(), "push");
    assert_eq!(push.to_string(), "push");
    assert_eq!(push.routing_key(), "notification.push");
    assert_eq!(push.queue_name(), "notification.push.queue");
    assert_eq!(push.dlq_name(), Some("notification.push.dlq"));

    let in_app = NotificationChannel::InApp;
    assert_eq!(in_app.as_str(), "in_app");
    assert_eq!(in_app.to_string(), "in_app");
    assert_eq!(in_app.routing_key(), "notification.in_app");
    assert_eq!(in_app.queue_name(), "notification.in_app.queue");
    assert_eq!(in_app.dlq_name(), None);

    assert_eq!(NotificationChannel::all().len(), 4);
    assert_eq!(
        NotificationChannel::from_str("email").unwrap(),
        NotificationChannel::Email
    );
    assert_eq!(
        NotificationChannel::from_str("whatsapp").unwrap(),
        NotificationChannel::Whatsapp
    );
    assert_eq!(
        NotificationChannel::from_str("push").unwrap(),
        NotificationChannel::Push
    );
    assert_eq!(
        NotificationChannel::from_str("in_app").unwrap(),
        NotificationChannel::InApp
    );
    assert!(NotificationChannel::from_str("sms").is_err());
}

#[test]
fn notification_status_parses_and_formats_supported_values() {
    assert_eq!(NotificationStatus::Pending.as_str(), "pending");
    assert_eq!(NotificationStatus::Sent.to_string(), "sent");
    assert_eq!(NotificationStatus::Failed.to_string(), "failed");
    assert_eq!(
        NotificationStatus::from_str("pending").unwrap(),
        NotificationStatus::Pending
    );
    assert!(NotificationStatus::from_str("unknown").is_err());
}

#[test]
fn app_error_maps_to_http_responses() {
    let validation = AppError::Validation("bad request".to_string()).into_response();
    assert_eq!(validation.status(), StatusCode::BAD_REQUEST);

    let internal = AppError::Internal("database unavailable".to_string()).into_response();
    assert_eq!(internal.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
