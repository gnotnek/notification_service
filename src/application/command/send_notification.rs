use crate::domain::notification_channel::NotificationChannel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub notification_id: Uuid,
    pub channel: NotificationChannel,
}
