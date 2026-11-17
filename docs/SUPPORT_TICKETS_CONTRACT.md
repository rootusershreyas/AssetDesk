# Member C — Support Tickets Contract

Status: Proposed; pending agreement with Members A and B.

## Ownership

- Member A: users and application foundation.
- Member B: categories, category-agent membership, shared database setup.
- Member C: tickets, ticket events, claiming, lifecycle, queries, and tests.

## Dependencies to confirm

- User role values and active-status fields.
- Category active-status fields and membership columns.
- Shared application state and JSON error format.
- Migration filenames and execution order.

## Proposed lifecycle

open -> in_progress -> awaiting_confirmation -> resolved

- Creation requires an active requester and a valid active category.
- Claiming requires an active, eligible IT agent and an unassigned open ticket.
- Only the assigned agent can submit nonempty resolution details.
- Only the original requester can confirm resolution.

## Transaction rules

- Every successful ticket mutation records an event in the same transaction.
- Two competing claims must yield one success and one conflict.
- Only one assignment and one claim event may be committed.
- Preserve ticket history; no hard deletion.

## Scope

Authentication, reopening, and administrative reassignment are extended work.
Request-supplied actor IDs are not authenticated identities.
## Proposed ticket fields

- id: UUID; unique ticket identifier
- requester_id: UUID referencing users
- category_id: UUID referencing categories
- assignee_id: optional UUID referencing users
- title: nonempty string
- description: nonempty string
- priority: low | medium | high
- status: open | in_progress | awaiting_confirmation | resolved
- resolution: optional string
- version: integer starting at 1
- created_at: timestamp with timezone
- updated_at: timestamp with timezone

Exact validation limits and reference columns require team agreement.

## Proposed ticket event fields

- id: UUID
- ticket_id: UUID referencing tickets
- actor_id: UUID referencing users
- event_type: created | claimed | resolution_submitted | confirmed
- from_status: optional ticket status
- to_status: ticket status
- created_at: timestamp with timezone

Events are append-only.

## Proposed endpoints

| Method | Route | Purpose |
| --- | --- | --- |
| POST | /api/v1/tickets | Create ticket |
| GET | /api/v1/tickets/{id} | Retrieve ticket |
| GET | /api/v1/tickets | List tickets and category queues |
| POST | /api/v1/tickets/{id}/claim | Claim ticket |
| POST | /api/v1/tickets/{id}/resolution | Submit resolution |
| POST | /api/v1/tickets/{id}/confirm | Confirm resolution |
| GET | /api/v1/tickets/{id}/events | Retrieve event history |

Creation accepts requester_id, category_id, title, description, and priority.
The server generates identity, timestamps, initial status, and version.

Lifecycle requests accept actor_id and expected_version.
Successful mutations increment version; stale versions return 409 Conflict.

## Proposed collection rules

- Filters: category_id, requester_id, assignee_id, status, priority.
- Default limit: 20; allowed range: 1–100.
- Offset: nonnegative integer, default 0.
- Initial sorting: created_at ascending or descending, with id as tie-breaker.
- Bind SQL values and allowlist sorting choices.

## Proposed HTTP responses

- 201: ticket created.
- 200: successful retrieval, listing, or lifecycle operation.
- 400: invalid input.
- 403: actor fails an eligibility or ownership rule.
- 404: referenced resource does not exist.
- 409: stale version, unavailable ticket, or invalid lifecycle transition.

Use the shared JSON error format agreed with Member A.