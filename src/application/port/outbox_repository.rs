use crate::shared::error::AppResult;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub retry_count: i32,
}

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn claim_pending(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>>;
    async fn mark_published(&self, id: Uuid) -> AppResult<()>;
    async fn mark_failed_for_retry(&self, id: Uuid) -> AppResult<()>;
}
