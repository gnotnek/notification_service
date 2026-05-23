use crate::{
    application::command::send_notification::NotificationMessage,
    domain::notification_channel::NotificationChannel, shared::error::AppResult,
};
use async_trait::async_trait;

#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(
        &self,
        channel: NotificationChannel,
        message: &NotificationMessage,
    ) -> AppResult<()>;
}
