# Operations

This document covers local setup and basic checks for running the service.

## Local Dependencies

The service needs:

- PostgreSQL
- RabbitMQ

You already have these locally. A `docker-compose.yml` is also available for a
fresh local setup.

```sh
docker compose up -d postgres rabbitmq
```

RabbitMQ management UI:

```text
http://localhost:15672
```

Default credentials:

```text
guest / guest
```

## Environment

Use `.env` for local values and `.env.example` as the shared template.

Current local defaults:

```sh
DATABASE_URL=postgres://postgres:postgres@localhost:5432/notification_service
RABBITMQ_URL=amqp://guest:guest@127.0.0.1:5672/%2f
HTTP_ADDR=127.0.0.1:3000
OUTBOX_BATCH_SIZE=25
PUBLISHER_INTERVAL_MS=1000
RUST_LOG=notification_service=info,sqlx=warn,lapin=warn
```

## Run the Service

```sh
cargo run
```

The service runs migrations at startup, declares RabbitMQ topology, starts the
outbox publisher, starts workers, and then serves the HTTP API.

## Create a Notification

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

Expected response:

```json
{
  "notification_id": "generated-uuid"
}
```

## Logs

Logs are emitted through `tracing` in JSON format. This makes them easier to
parse by log collectors and easier to filter by field.

Useful log fields include:

- `notification_id`
- `outbox_id`
- `channel`
- `routing_key`
- `error`

## Basic Database Checks

Recent notifications:

```sql
SELECT id, recipient, channel, status, created_at, updated_at
FROM notifications
ORDER BY created_at DESC
LIMIT 10;
```

Recent outbox events:

```sql
SELECT id, notification_id, event_type, status, retry_count, published_at, created_at
FROM notification_outbox
ORDER BY created_at DESC
LIMIT 10;
```

Recent attempts:

```sql
SELECT notification_id, channel, status, provider_response, error_message, attempted_at
FROM notification_attempts
ORDER BY attempted_at DESC
LIMIT 10;
```

## Basic RabbitMQ Checks

In the RabbitMQ management UI, check:

- Exchange `notification.exchange` exists.
- Channel queues exist and are bound with their routing keys.
- DLQs exist for email, WhatsApp, and push.
- Messages move from queues after workers process them.

## Verification

Run formatting and tests:

```sh
cargo fmt --check
cargo test
```

Run the unit coverage gate:

```sh
cargo llvm-cov --summary-only --ignore-filename-regex '(^|/)(tests/|src/(app|main|config)\.rs|src/infrastructure/(postgres|rabbitmq)/|src/interface/worker/|src/application/service/outbox_publisher\.rs|src/shared/logger\.rs)' --fail-under-lines 90
```

The unit coverage gate excludes startup wiring, PostgreSQL/RabbitMQ adapters,
and forever-running worker loops. Those boundaries need integration tests
against real local services.
