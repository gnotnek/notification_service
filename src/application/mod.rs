pub mod command {
    pub mod create_notification;
    pub mod send_notification;
}

pub mod port {
    pub mod message_broker;
    pub mod notification_repository;
    pub mod notification_sender;
    pub mod outbox_repository;
}

pub mod service {
    pub mod notification_service;
    pub mod outbox_publisher;
}
