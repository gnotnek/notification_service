use crate::application::service::outbox_publisher::OutboxPublisher;
use std::{sync::Arc, time::Duration};

pub fn spawn_outbox_worker(publisher: Arc<OutboxPublisher>, interval: Duration) {
    tokio::spawn(async move {
        publisher.run(interval).await;
    });
}
