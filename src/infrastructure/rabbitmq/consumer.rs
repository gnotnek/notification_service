use crate::{infrastructure::rabbitmq::publisher::declare_topology, shared::error::AppResult};
use lapin::{
    Channel, Connection, ConnectionProperties,
    options::{BasicConsumeOptions, BasicQosOptions},
    types::FieldTable,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RabbitMqConsumer {
    _connection: Arc<Connection>,
    channel: Channel,
}

impl RabbitMqConsumer {
    pub async fn connect(url: &str, prefetch_count: u16) -> AppResult<Self> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;
        declare_topology(&channel).await?;
        channel
            .basic_qos(prefetch_count, BasicQosOptions::default())
            .await?;

        Ok(Self {
            _connection: Arc::new(connection),
            channel,
        })
    }

    pub async fn consume(&self, queue: &str, consumer_tag: &str) -> AppResult<lapin::Consumer> {
        Ok(self
            .channel
            .basic_consume(
                queue,
                consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?)
    }
}
