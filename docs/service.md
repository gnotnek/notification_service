# Service

The notification service exposes an HTTP API and starts background tasks for
outbox publishing and RabbitMQ workers.

## API

### Create Notification

```http
POST /notifications
content-type: application/json
```

Request body:

```json
{
  "recipient": "user@example.com",
  "channel": "email",
  "subject": "Welcome",
  "body": "Your account is ready",
  "payload": {
    "template": "welcome"
  },
  "scheduled_at": null
}
```

Response:

```json
{
  "notification_id": "9a1e5fb1-71a9-44f8-9b51-b25e21c21157"
}
```

## Supported Channels

- `email`
- `whatsapp`
- `push`
- `in_app`

## Validation

The API currently validates:

- `recipient` must not be empty.
- `body` must not be empty.
- `channel` must be one of the supported channel values.

## Runtime Flow

### 1. Create Notification

```text
POST /notifications
  -> validate request
  -> insert into notifications
  -> insert into notification_outbox
  -> return notification_id
```

The `notifications` row starts with status `pending`.

The `notification_outbox` row starts with status `pending` and contains a JSON
payload like:

```json
{
  "notification_id": "9a1e5fb1-71a9-44f8-9b51-b25e21c21157",
  "channel": "email"
}
```

### 2. Outbox Publisher

```text
loop:
  -> find pending outbox rows
  -> publish to RabbitMQ
  -> mark outbox as published
```

The publisher claims rows by updating their status to `processing`. This avoids
multiple publisher loops working on the same row at the same time.

If RabbitMQ publish succeeds, the row becomes `published`.

If RabbitMQ publish fails, the row becomes `pending` again unless the retry
limit has been reached. The next retry is delayed by 10 seconds.

### 3. Worker

```text
consume RabbitMQ message
  -> parse notification_id
  -> load notification
  -> send notification using provider
  -> insert notification_attempt
  -> update notification status
  -> ACK if success
  -> NACK/retry if failed
```

On provider success:

- Insert a `notification_attempts` row with status `success`.
- Update `notifications.status` to `sent`.
- ACK the RabbitMQ message.

On provider failure:

- Insert a `notification_attempts` row with status `failed`.
- NACK and requeue the message once.
- If the redelivered message fails again, update `notifications.status` to
  `failed` and NACK without requeue. RabbitMQ routes it to a dead-letter queue
  when the queue has one configured.

## Configuration

The service reads configuration from environment variables. `.env` is loaded in
local development.

| Variable | Default | Description |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/notification_service` | PostgreSQL connection string. |
| `RABBITMQ_URL` | `amqp://guest:guest@127.0.0.1:5672/%2f` | RabbitMQ AMQP connection string. |
| `HTTP_ADDR` | `127.0.0.1:3000` | API bind address. |
| `OUTBOX_BATCH_SIZE` | `25` | Number of outbox rows claimed per publisher loop. |
| `PUBLISHER_INTERVAL_MS` | `1000` | Sleep duration between publisher loops. |
| `RUST_LOG` | `notification_service=info` | Log filtering for `tracing`. |
| `RESEND_API_KEY` | none | Required for the email worker. |
| `RESEND_FROM_EMAIL` | none | Required sender address for Resend, for example `Notification Service <notifications@example.com>`. |
