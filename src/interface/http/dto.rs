use crate::domain::notification_channel::NotificationChannel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub recipient: String,
    pub channel: NotificationChannel,
    pub subject: Option<String>,
    pub body: String,
    pub payload: Option<Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateNotificationResponse {
    pub notification_id: Uuid,
}
