use crate::{
    application::command::create_notification::CreateNotificationCommand,
    interface::http::dto::{CreateNotificationRequest, CreateNotificationResponse},
    shared::error::AppResult,
};
use axum::{Json, extract::State, http::StatusCode};

use super::route::HttpState;

pub async fn create_notification(
    State(state): State<HttpState>,
    Json(request): Json<CreateNotificationRequest>,
) -> AppResult<(StatusCode, Json<CreateNotificationResponse>)> {
    let result = state
        .notification_service
        .create(CreateNotificationCommand {
            recipient: request.recipient,
            channel: request.channel,
            subject: request.subject,
            body: request.body,
            payload: request.payload,
            scheduled_at: request.scheduled_at,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateNotificationResponse {
            notification_id: result.notification_id,
        }),
    ))
}
