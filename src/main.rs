#[tokio::main]
async fn main() {
    notification_service::shared::logger::init();

    if let Err(err) = notification_service::app::run().await {
        tracing::error!(error = %err, "notification service stopped");
        std::process::exit(1);
    }
}
