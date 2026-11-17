# AssetDesk – IT Infrastructure Ticket Management System

An IT support ticket management API using Rust, Axum, Tokio, SQLx, and PostgreSQL.

Employees create tickets, eligible IT agents claim category-based work, and requesters confirm resolutions. Ticket changes preserve transactional event history.

## Current status

Planning and foundation development. This branch contains Member C's proposed contract; ticket functionality is not implemented yet.

## Team ownership

- Member A: users and application foundation.
- Member B: categories, category-agent membership, and shared database setup.
- Member C: tickets, claiming, lifecycle, event history, collection queries, and concurrency tests.

## Member C workflow

Work on `feat/support-tickets`. See [the proposed ticket contract](docs/SUPPORT_TICKETS_CONTRACT.md).

Implement and test small tasks, then commit and push each completed milestone.

## Prototype scope

- Five business tables: users, categories, category_agents, tickets, and ticket_events.
- Category queues and eligible-agent claiming.
- Four-state ticket lifecycle and transactional event history.
- Filtering, sorting, pagination, OpenAPI documentation, and automated tests.

Authentication, reopening, and administrative reassignment are extended work. The initial prototype is for local development and demonstration.
