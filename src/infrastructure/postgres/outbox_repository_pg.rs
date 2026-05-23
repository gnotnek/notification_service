use crate::{
    application::port::outbox_repository::{OutboxEvent, OutboxRepository},
    shared::error::AppResult,
};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresOutboxRepository {
    pool: PgPool,
}

impl PostgresOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxRepository for PostgresOutboxRepository {
    async fn claim_pending(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>> {
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            UPDATE notification_outbox
            SET status = 'processing'
            WHERE id IN (
                SELECT id
                FROM notification_outbox
                WHERE status = 'pending'
                  AND (next_retry_at IS NULL OR next_retry_at <= now())
                ORDER BY created_at
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, notification_id, event_type, payload, retry_count
            "#,
        )
        .bind(batch_size)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_published(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notification_outbox
            SET status = 'published', published_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_failed_for_retry(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notification_outbox
            SET
                retry_count = retry_count + 1,
                status = CASE
                    WHEN retry_count + 1 >= max_retry THEN 'failed'
                    ELSE 'pending'
                END,
                next_retry_at = CASE
                    WHEN retry_count + 1 >= max_retry THEN NULL
                    ELSE now() + interval '10 seconds'
                END
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct OutboxEventRow {
    id: Uuid,
    notification_id: Uuid,
    event_type: String,
    payload: Json<Value>,
    retry_count: i32,
}

impl From<OutboxEventRow> for OutboxEvent {
    fn from(row: OutboxEventRow) -> Self {
        Self {
            id: row.id,
            notification_id: row.notification_id,
            event_type: row.event_type,
            payload: row.payload.0,
            retry_count: row.retry_count,
        }
    }
}
