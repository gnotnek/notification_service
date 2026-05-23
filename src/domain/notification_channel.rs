use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Whatsapp,
    Push,
    InApp,
}

impl NotificationChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Whatsapp => "whatsapp",
            Self::Push => "push",
            Self::InApp => "in_app",
        }
    }

    pub fn routing_key(self) -> &'static str {
        match self {
            Self::Email => "notification.email",
            Self::Whatsapp => "notification.whatsapp",
            Self::Push => "notification.push",
            Self::InApp => "notification.in_app",
        }
    }

    pub fn queue_name(self) -> &'static str {
        match self {
            Self::Email => "notification.email.queue",
            Self::Whatsapp => "notification.whatsapp.queue",
            Self::Push => "notification.push.queue",
            Self::InApp => "notification.in_app.queue",
        }
    }

    pub fn dlq_name(self) -> Option<&'static str> {
        match self {
            Self::Email => Some("notification.email.dlq"),
            Self::Whatsapp => Some("notification.whatsapp.dlq"),
            Self::Push => Some("notification.push.dlq"),
            Self::InApp => None,
        }
    }

    pub fn dlq_routing_key(self) -> Option<&'static str> {
        match self {
            Self::Email => Some("notification.email.dlq"),
            Self::Whatsapp => Some("notification.whatsapp.dlq"),
            Self::Push => Some("notification.push.dlq"),
            Self::InApp => None,
        }
    }

    pub fn all() -> [Self; 4] {
        [Self::Email, Self::Whatsapp, Self::Push, Self::InApp]
    }
}

impl fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NotificationChannel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "email" => Ok(Self::Email),
            "whatsapp" => Ok(Self::Whatsapp),
            "push" => Ok(Self::Push),
            "in_app" => Ok(Self::InApp),
            other => Err(format!("unsupported notification channel: {other}")),
        }
    }
}
