# Architecture

The service follows an outbox-based notification architecture. The API writes
both the notification request and a matching outbox event in the same PostgreSQL
transaction. A background publisher later reads pending outbox events and sends
them to RabbitMQ. Notification workers consume RabbitMQ messages and call the
channel provider.

## High-Level Flow

```text
Client/API
  |
  v
Notification API
  |
  | 1. Save notification request
  | 2. Save outbox event in PostgreSQL
  v
PostgreSQL
  |
  | background publisher reads pending outbox
  v
RabbitMQ
  |
  v
Notification Workers
  |
  +--> Email provider
  +--> WhatsApp provider
  +--> Push notification provider
  +--> In-app notification
```

## Components

### Notification API

The API accepts `POST /notifications` requests. It validates required fields,
creates a row in `notifications`, creates a row in `notification_outbox`, and
returns the generated `notification_id`.

The API does not call providers directly. That keeps request latency low and
keeps delivery work resilient if RabbitMQ or a provider has temporary issues.

### PostgreSQL

PostgreSQL is the source of truth for notification requests, delivery state,
outbox events, and delivery attempts.

The important reliability rule is that `notifications` and
`notification_outbox` are inserted together in one database transaction. If the
transaction succeeds, the notification request is safely stored and can be
published later.

### Outbox Publisher

The outbox publisher is a background task inside the service process. It claims
pending outbox rows, publishes the message to RabbitMQ, and marks the outbox row
as `published`.

If publishing fails, the row is returned to `pending` with a retry delay until
its retry limit is reached.

### RabbitMQ

RabbitMQ routes notification messages by channel. The exchange is direct, and
each channel has a routing key and queue.

Email, WhatsApp, and push queues also have dead-letter queues. In-app currently
does not have a dead-letter queue because the original design did not include
one.

### Notification Workers

Workers consume messages from RabbitMQ queues. Each message contains the
`notification_id` and target channel.

The worker loads the notification from PostgreSQL, calls the matching provider,
records an attempt, updates the notification status, and acknowledges or rejects
the RabbitMQ message.

### Providers

Email uses Resend through the `resend-rs` client. The other providers are
currently mocks:

- `ResendEmailSender`
- `MockWhatsappSender`
- `MockPushSender`
- `MockInAppSender`

They return fake provider responses and are intended to be replaced later with
real integrations.

## Reliability Model

The service uses the outbox pattern to avoid losing notification requests after
the API returns successfully.

The API is responsible for durable writes. The publisher is responsible for
moving durable outbox rows into RabbitMQ. The worker is responsible for delivery
attempts and final status updates.

## Known Tradeoffs

- Workers run in the same binary as the API. This is simple for local
  development, but production deployments may split API, publisher, and workers
  into separate processes.
- Provider implementations are mocks, so real provider-specific retry rules,
  authentication, and response mapping are not implemented yet.
- Only email, WhatsApp, and push have dead-letter queues because that was the
  requested queue design.
