# Notification Service Documentation

This folder explains the notification service in human-readable terms. The
service receives notification requests over HTTP, stores them in PostgreSQL,
publishes delivery work through RabbitMQ, and processes each channel with worker
tasks.

## Documents

- [Architecture](architecture.md): high-level system design and component
  responsibilities.
- [Service](service.md): API behavior, runtime flow, worker behavior, and
  configuration.
- [Data Catalog](data-catalog.md): database tables, important columns, and data
  lifecycle.
- [Queue Design](queue-design.md): RabbitMQ exchange, routing keys, queues, and
  dead-letter behavior.
- [Operations](operations.md): local setup, running, logging, and useful checks.

