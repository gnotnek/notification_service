use crate::{
    application::{
        command::{
            create_notification::CreateNotificationCommand, send_notification::NotificationMessage,
        },
        port::notification_repository::NotificationRepository,
    },
    domain::{
        notification::Notification, notification_channel::NotificationChannel,
        notification_status::NotificationStatus,
    },
    shared::error::{AppError, AppResult},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, types::Json};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create_with_outbox(&self, command: CreateNotificationCommand) -> AppResult<Uuid> {
        let notification_id = Uuid::new_v4();
        let outbox_id = Uuid::new_v4();
        let channel = command.channel;
        let outbox_payload = NotificationMessage {
            notification_id,
            channel,
        };

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO notifications (
                id, recipient, channel, subject, body, payload, status, scheduled_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)
            "#,
        )
        .bind(notification_id)
        .bind(command.recipient.trim())
        .bind(channel.as_str())
        .bind(command.subject.as_deref())
        .bind(command.body.trim())
        .bind(command.payload.map(Json))
        .bind(command.scheduled_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO notification_outbox (
                id, notification_id, event_type, payload, status
            )
            VALUES ($1, $2, 'notification.created', $3, 'pending')
            "#,
        )
        .bind(outbox_id)
        .bind(notification_id)
        .bind(Json(json!(outbox_payload)))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(notification_id)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Notification>> {
        let row = sqlx::query_as::<_, NotificationRow>(
            r#"
            SELECT
                id, recipient, channel, subject, body, payload, status, scheduled_at,
                sent_at, failed_at, created_at, updated_at
            FROM notifications
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(NotificationRow::try_into_notification).transpose()
    }

    async fn mark_sent(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET status = 'sent', sent_at = now(), failed_at = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_failed(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET status = 'failed', failed_at = now(), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn record_attempt(
        &self,
        notification: &Notification,
        status: &str,
        provider_response: Option<Value>,
        error_message: Option<String>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO notification_attempts (
                id, notification_id, channel, status, provider_response, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(notification.id)
        .bind(notification.channel.as_str())
        .bind(status)
        .bind(provider_response.map(Json))
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct NotificationRow {
    id: Uuid,
    recipient: String,
    channel: String,
    subject: Option<String>,
    body: String,
    payload: Option<Json<Value>>,
    status: String,
    scheduled_at: Option<DateTime<Utc>>,
    sent_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl NotificationRow {
    fn try_into_notification(self) -> AppResult<Notification> {
        Ok(Notification {
            id: self.id,
            recipient: self.recipient,
            channel: NotificationChannel::from_str(&self.channel).map_err(AppError::Validation)?,
            subject: self.subject,
            body: self.body,
            payload: self.payload.map(|value| value.0),
            status: NotificationStatus::from_str(&self.status).map_err(AppError::Validation)?,
            scheduled_at: self.scheduled_at,
            sent_at: self.sent_at,
            failed_at: self.failed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
