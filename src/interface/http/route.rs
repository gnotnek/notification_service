use crate::application::service::notification_service::NotificationService;
use axum::{Router, routing::post};
use std::sync::Arc;

use super::handler::create_notification;

#[derive(Clone)]
pub struct HttpState {
    pub notification_service: Arc<NotificationService>,
}

pub fn router(state: HttpState) -> Router {
    Router::new()
        .route("/notifications", post(create_notification))
        .with_state(state)
}
