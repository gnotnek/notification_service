# Data Catalog

PostgreSQL stores notification state, outbox events, and delivery attempts.

## Table: `notifications`

Stores the user-facing notification request and its current delivery status.

| Column | Type | Description |
| --- | --- | --- |
| `id` | `UUID` | Primary key returned to the API caller. |
| `recipient` | `TEXT` | Target recipient. Format depends on channel. |
| `channel` | `TEXT` | One of `email`, `whatsapp`, `push`, `in_app`. |
| `subject` | `TEXT` | Optional subject. Mainly useful for email. |
| `body` | `TEXT` | Notification body. |
| `payload` | `JSONB` | Optional structured metadata for templates or provider-specific fields. |
| `status` | `TEXT` | Current state: `pending`, `sent`, or `failed`. |
| `scheduled_at` | `TIMESTAMPTZ` | Optional future delivery time. Stored but not enforced by current worker logic. |
| `sent_at` | `TIMESTAMPTZ` | Time when the notification was marked sent. |
| `failed_at` | `TIMESTAMPTZ` | Time when the notification was marked failed. |
| `created_at` | `TIMESTAMPTZ` | Creation timestamp. |
| `updated_at` | `TIMESTAMPTZ` | Last status update timestamp. |

Indexes:

- `idx_notifications_status`
- `idx_notifications_channel`

## Table: `notification_outbox`

Stores durable events that need to be published to RabbitMQ.

| Column | Type | Description |
| --- | --- | --- |
| `id` | `UUID` | Primary key for the outbox event. |
| `notification_id` | `UUID` | Related `notifications.id`. |
| `event_type` | `TEXT` | Current event type is `notification.created`. |
| `payload` | `JSONB` | Message body published to RabbitMQ. |
| `status` | `TEXT` | Current state: `pending`, `processing`, `published`, or `failed`. |
| `retry_count` | `INT` | Number of failed publish attempts. |
| `max_retry` | `INT` | Maximum publish retry attempts. Default is `5`. |
| `next_retry_at` | `TIMESTAMPTZ` | Earliest time this row can be retried. |
| `published_at` | `TIMESTAMPTZ` | Time when the row was successfully published. |
| `created_at` | `TIMESTAMPTZ` | Creation timestamp. |

Indexes:

- `idx_notification_outbox_pending`

## Table: `notification_attempts`

Stores each worker delivery attempt.

| Column | Type | Description |
| --- | --- | --- |
| `id` | `UUID` | Primary key for the attempt. |
| `notification_id` | `UUID` | Related `notifications.id`. |
| `channel` | `TEXT` | Channel used for the attempt. |
| `status` | `TEXT` | Attempt result: `success` or `failed`. |
| `provider_response` | `JSONB` | Provider response payload for successful sends. |
| `error_message` | `TEXT` | Error text for failed sends. |
| `attempted_at` | `TIMESTAMPTZ` | Attempt timestamp. |

Indexes:

- `idx_notification_attempts_notification_id`

## Data Lifecycle

1. API creates `notifications` with status `pending`.
2. API creates `notification_outbox` with status `pending`.
3. Publisher changes outbox status to `processing`.
4. Publisher sends to RabbitMQ.
5. Publisher changes outbox status to `published`.
6. Worker inserts `notification_attempts`.
7. Worker updates notification status to `sent` or `failed`.

## Status Meanings

### Notification Status

| Status | Meaning |
| --- | --- |
| `pending` | Request was accepted but delivery has not completed. |
| `sent` | Provider mock accepted or stored the notification. |
| `failed` | Worker could not deliver the notification after retry behavior. |

### Outbox Status

| Status | Meaning |
| --- | --- |
| `pending` | Event is ready to publish. |
| `processing` | Publisher claimed the event. |
| `published` | Event was published to RabbitMQ. |
| `failed` | Event exceeded publish retry limit. |

