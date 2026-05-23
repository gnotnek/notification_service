# Queue Design

RabbitMQ is used to route notification work to channel-specific workers.

## Exchange

| Name | Type | Durable |
| --- | --- | --- |
| `notification.exchange` | `direct` | Yes |

The exchange is declared by the application when it starts.

## Routing Keys

| Channel | Routing Key |
| --- | --- |
| Email | `notification.email` |
| WhatsApp | `notification.whatsapp` |
| Push | `notification.push` |
| In-app | `notification.in_app` |

## Queues

| Channel | Queue | Routing Key |
| --- | --- | --- |
| Email | `notification.email.queue` | `notification.email` |
| WhatsApp | `notification.whatsapp.queue` | `notification.whatsapp` |
| Push | `notification.push.queue` | `notification.push` |
| In-app | `notification.in_app.queue` | `notification.in_app` |

All queues are durable.

## Dead-Letter Queues

| Source Queue | Dead-Letter Queue | Dead-Letter Routing Key |
| --- | --- | --- |
| `notification.email.queue` | `notification.email.dlq` | `notification.email.dlq` |
| `notification.whatsapp.queue` | `notification.whatsapp.dlq` | `notification.whatsapp.dlq` |
| `notification.push.queue` | `notification.push.dlq` | `notification.push.dlq` |

The in-app queue does not have a dead-letter queue in the current design.

## Message Payload

Workers receive JSON messages with this shape:

```json
{
  "notification_id": "9a1e5fb1-71a9-44f8-9b51-b25e21c21157",
  "channel": "email"
}
```

The worker uses `notification_id` to load the full notification from
PostgreSQL. Keeping the RabbitMQ message small avoids duplicating notification
content across systems.

## Failure Behavior

If a worker cannot parse a message, it ACKs the message and logs a warning. This
prevents poison messages from blocking a queue forever.

If a provider fails:

1. The first failure is NACKed with requeue enabled.
2. If the redelivered message fails again, it is NACKed without requeue.
3. RabbitMQ sends it to the configured dead-letter queue when the source queue
   has dead-letter settings.

