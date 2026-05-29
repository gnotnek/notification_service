use crate::{
    application::{
        port::{
            message_broker::MessageBroker, notification_repository::NotificationRepository,
            outbox_repository::OutboxRepository,
        },
        service::{notification_service::NotificationService, outbox_publisher::OutboxPublisher},
    },
    config::AppConfig,
    infrastructure::{
        postgres::{
            notification_repository_pg::PostgresNotificationRepository,
            outbox_repository_pg::PostgresOutboxRepository,
        },
        rabbitmq::publisher::RabbitMqPublisher,
    },
    interface::{
        http::route::{HttpState, router},
        worker::{notification_worker::NotificationWorker, outbox_worker::spawn_outbox_worker},
    },
    shared::error::{AppError, AppResult},
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;

pub async fn run() -> AppResult<()> {
    let config = AppConfig::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| AppError::Internal(format!("failed to run database migrations: {err}")))?;

    let notification_repository = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let outbox_repository: Arc<dyn OutboxRepository> =
        Arc::new(PostgresOutboxRepository::new(pool));
    let rabbitmq_publisher: Arc<dyn MessageBroker> =
        Arc::new(RabbitMqPublisher::connect(&config.rabbitmq_url).await?);

    let outbox_publisher = Arc::new(OutboxPublisher::new(
        outbox_repository,
        rabbitmq_publisher,
        config.outbox_batch_size,
    ));
    spawn_outbox_worker(outbox_publisher, config.publisher_interval);

    let notification_worker_repository: Arc<dyn NotificationRepository> =
        notification_repository.clone();
    Arc::new(NotificationWorker::new(
        notification_worker_repository,
        &config.resend_api_key,
        &config.resend_from_email,
    )?)
    .run_all(config.rabbitmq_url.clone())
    .await?;

    let notification_service_repository: Arc<dyn NotificationRepository> = notification_repository;
    let notification_service = Arc::new(NotificationService::new(notification_service_repository));
    let app = router(HttpState {
        notification_service,
    });

    let listener = tokio::net::TcpListener::bind(config.http_addr)
        .await
        .map_err(|err| AppError::Internal(format!("failed to bind HTTP listener: {err}")))?;

    info!(addr = %config.http_addr, "notification api listening");

    axum::serve(listener, app)
        .await
        .map_err(|err| AppError::Internal(format!("http server failed: {err}")))
}
