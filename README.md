# Notification Service

Rust notification service using Axum, PostgreSQL, RabbitMQ, SQLx, Lapin, and
`tracing`.

More human-readable project docs are available in [docs/README.md](docs/README.md).

## Runtime Flow

1. `POST /notifications` validates the request, inserts `notifications`, inserts
   `notification_outbox`, and returns `notification_id`.
2. The outbox publisher claims pending outbox rows, publishes to RabbitMQ, and
   marks rows as published.
3. Notification workers consume RabbitMQ messages, call mock providers, insert
   `notification_attempts`, update notification status, and ACK/NACK messages.

## Queue Topology

- Exchange: `notification.exchange`
- Routing keys: `notification.email`, `notification.whatsapp`,
  `notification.push`, `notification.in_app`
- Queues: `notification.email.queue`, `notification.whatsapp.queue`,
  `notification.push.queue`, `notification.in_app.queue`
- Dead-letter queues: `notification.email.dlq`, `notification.whatsapp.dlq`,
  `notification.push.dlq`

## Environment

Defaults are suitable for the included `docker-compose.yml`.

```sh
DATABASE_URL=postgres://postgres:postgres@localhost:5432/notification_service
RABBITMQ_URL=amqp://guest:guest@127.0.0.1:5672/%2f
HTTP_ADDR=127.0.0.1:3000
OUTBOX_BATCH_SIZE=25
PUBLISHER_INTERVAL_MS=1000
RUST_LOG=notification_service=info
RESEND_API_KEY=re_your_api_key_here
RESEND_FROM_EMAIL="Notification Service <notifications@example.com>"
```

## Run

```sh
cargo run
```

## Test and Coverage

```sh
cargo test
cargo llvm-cov --summary-only --ignore-filename-regex '(^|/)(tests/|src/(app|main|config)\.rs|src/infrastructure/(postgres|rabbitmq)/|src/interface/worker/|src/application/service/outbox_publisher\.rs|src/shared/logger\.rs)' --fail-under-lines 90
```

The coverage gate focuses on unit-testable application/domain/interface logic.
Startup wiring, external PostgreSQL/RabbitMQ adapters, and forever-running
worker loops should be covered separately by integration tests with real
services.

Example request:

```sh
curl -X POST http://127.0.0.1:3000/notifications \
  -H 'content-type: application/json' \
  -d '{
    "recipient": "user@example.com",
    "channel": "email",
    "subject": "Welcome",
    "body": "Your account is ready",
    "payload": {"template": "welcome"}
  }'
```
