use crate::{
    application::{
        command::send_notification::NotificationMessage, port::message_broker::MessageBroker,
    },
    domain::notification_channel::NotificationChannel,
    shared::error::AppResult,
};
use async_trait::async_trait;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable, LongString, ShortString},
};
use std::sync::Arc;
use tracing::info;

pub const EXCHANGE_NAME: &str = "notification.exchange";

pub struct RabbitMqPublisher {
    _connection: Arc<Connection>,
    channel: Channel,
}

impl RabbitMqPublisher {
    pub async fn connect(url: &str) -> AppResult<Self> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;
        declare_topology(&channel).await?;
        Ok(Self {
            _connection: Arc::new(connection),
            channel,
        })
    }
}

#[async_trait]
impl MessageBroker for RabbitMqPublisher {
    async fn publish(
        &self,
        notification_channel: NotificationChannel,
        message: &NotificationMessage,
    ) -> AppResult<()> {
        let payload = serde_json::to_vec(message)?;

        self.channel
            .basic_publish(
                EXCHANGE_NAME,
                notification_channel.routing_key(),
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2),
            )
            .await?
            .await?;

        Ok(())
    }
}

pub async fn declare_topology(channel: &Channel) -> AppResult<()> {
    channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    for notification_channel in NotificationChannel::all() {
        if let (Some(dlq_name), Some(dlq_routing_key)) = (
            notification_channel.dlq_name(),
            notification_channel.dlq_routing_key(),
        ) {
            channel
                .queue_declare(
                    dlq_name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;
            channel
                .queue_bind(
                    dlq_name,
                    EXCHANGE_NAME,
                    dlq_routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        let mut queue_args = FieldTable::default();
        if let Some(dlq_routing_key) = notification_channel.dlq_routing_key() {
            queue_args.insert(
                ShortString::from("x-dead-letter-exchange"),
                AMQPValue::LongString(LongString::from(EXCHANGE_NAME.as_bytes())),
            );
            queue_args.insert(
                ShortString::from("x-dead-letter-routing-key"),
                AMQPValue::LongString(LongString::from(dlq_routing_key.as_bytes())),
            );
        }

        channel
            .queue_declare(
                notification_channel.queue_name(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                queue_args,
            )
            .await?;
        channel
            .queue_bind(
                notification_channel.queue_name(),
                EXCHANGE_NAME,
                notification_channel.routing_key(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
    }

    info!("rabbitmq notification topology declared");
    Ok(())
}
