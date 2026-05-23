use crate::shared::error::{AppError, AppResult};
use std::{env, net::SocketAddr, time::Duration};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: SocketAddr,
    pub database_url: String,
    pub rabbitmq_url: String,
    pub outbox_batch_size: i64,
    pub publisher_interval: Duration,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        dotenvy::dotenv().ok();

        let http_addr = env::var("HTTP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
            .parse()
            .map_err(|err| AppError::Config(format!("invalid HTTP_ADDR: {err}")))?;

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/notification_service".to_string()
        });

        let rabbitmq_url = env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".to_string());

        let outbox_batch_size = env::var("OUTBOX_BATCH_SIZE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(25);

        let publisher_interval_ms = env::var("PUBLISHER_INTERVAL_MS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1_000);

        Ok(Self {
            http_addr,
            database_url,
            rabbitmq_url,
            outbox_batch_size,
            publisher_interval: Duration::from_millis(publisher_interval_ms),
        })
    }
}
