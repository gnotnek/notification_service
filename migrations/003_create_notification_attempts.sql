CREATE TABLE IF NOT EXISTS notification_attempts (
    id UUID PRIMARY KEY,
    notification_id UUID NOT NULL REFERENCES notifications(id),
    channel TEXT NOT NULL,
    status TEXT NOT NULL,
    provider_response JSONB,
    error_message TEXT,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_notification_attempts_notification_id
ON notification_attempts(notification_id);
