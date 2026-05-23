use crate::{
    application::{
        command::create_notification::{CreateNotificationCommand, CreateNotificationResult},
        port::notification_repository::NotificationRepository,
    },
    shared::error::{AppError, AppResult},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationService {
    repository: Arc<dyn NotificationRepository>,
}

impl NotificationService {
    pub fn new(repository: Arc<dyn NotificationRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        command: CreateNotificationCommand,
    ) -> AppResult<CreateNotificationResult> {
        validate(&command)?;
        let notification_id = self.repository.create_with_outbox(command).await?;
        Ok(CreateNotificationResult { notification_id })
    }
}

fn validate(command: &CreateNotificationCommand) -> AppResult<()> {
    if command.recipient.trim().is_empty() {
        return Err(AppError::Validation("recipient is required".to_string()));
    }

    if command.body.trim().is_empty() {
        return Err(AppError::Validation("body is required".to_string()));
    }

    Ok(())
}
