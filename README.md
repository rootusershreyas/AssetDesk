# AssetDesk — IT Infrastructure Ticket Management

AssetDesk helps organizations manage employee IT issues and service requests through a centralized platform. Employees raise tickets, IT Managers control assignment, and eligible IT Team Members troubleshoot assigned work. The application records progress, resolution details and ticket history. The complete application also provides persistent conversations and optional image attachments.

This README records the updated implementation context supplied on 18 November 2026. Its scope and business rules take precedence where older planning documents disagree. Features below are requirements unless the implementation progress section explicitly records completed work.

## Roles and responsibilities

| Role | Responsibilities |
| --- | --- |
| Employee | Create Service Requests and Incidents, select category and priority, describe the issue, view own tickets and confirm resolution. The extended application adds images, conversations and reopening. |
| IT Manager | Review incoming tickets, personally handle P1, assign P2–P4 to eligible members, monitor progress and review history and conversations. |
| IT Team Member | View assigned work, investigate, record progress and resolution, and submit the result for employee confirmation. The extended application enables conversation with the requester and access to previous messages on assigned tickets. |

An employee can view their own requests. A member's category eligibility permits assignment but does not grant access to every ticket in that category. Backend access checks enforce the current ticket relationship. Authentication and validated sessions are a later increment; prototype actor fixtures are limited to local testing.

## Ticket types and categories

| Type | Purpose | Example categories |
| --- | --- | --- |
| Service Request (`service_request`) | Request an IT service or resource | Software Installation, Hardware Request, Access Permission, New Account, System Configuration |
| Incident (`incident`) | Report an unexpected IT disruption | Network Issue, Internet Connectivity, VPN Issue, Application Error, Login Issue, System Performance Issue, Others |

Incident categories exclude Hardware and Hardware Issue. Hardware Request belongs under Service Request. Incident must include an Others option.

Selecting Incident Others requires a nonblank `custom_concern` on the ticket. Accept valid free text without requiring it to match a predefined category. Store it separately from the general `description`. B and C must agree a stable way to identify the Others category and a length limit for `custom_concern` before integrating that validation. A frontend displays the field when Others is selected.

Category names are trimmed, contain 1–80 characters and are unique ignoring case within their ticket type. Category descriptions are optional and limited to 1,000 characters. Preserve referenced categories through deactivation. Category type is immutable after creation.

## Ticket creation and assignment

1. The employee selects ticket type, category and priority P1–P4.
2. The employee enters a mandatory description and, for Incident Others, a custom concern.
3. The system generates an internal UUID and an immutable SR/INC ticket reference, records creation and places the ticket in the IT Manager's queue.
4. The manager reviews the submitted information and takes responsibility for P1 or assigns P2–P4 to an eligible IT Team Member.
5. The handler investigates, records progress and submits resolution details for employee confirmation.

Descriptions must contain non-whitespace text and at most 1,000 characters. A single meaningful character is valid; there is no additional minimum-length requirement. The future UI should display a character counter. Backend validation enforces the limit independently.

Priority comes from the employee at creation. The manager controls assignment. Enforce these rules in the backend:

- P1 must be handled personally by the IT Manager and cannot be assigned to an IT Team Member.
- Only the IT Manager can assign P2, P3 and P4.
- The selected IT Team Member must be active and eligible for the category.
- IT Team Members cannot claim tickets or assign tickets to themselves.
- Ticket updates and the corresponding audit events commit together in a transaction.
- Concurrent changes must respect the agreed version and eligibility checks.

## Lifecycle and confirmation

The four user-visible stages for P6 are:

```text
Open -> In Progress -> Awaiting Employee Confirmation -> Resolved
```

The current planning contract uses API values `open`, `in_progress`, `resolved` and `closed`. Under that mapping, API `resolved` is displayed as Awaiting Employee Confirmation, and API `closed` is displayed as Resolved after requester confirmation. This README retains those API values for compatibility. A, B and C must review any change to their serialized values before modifying Rust enums or SQL constraints.

Only the original requester confirms the final outcome, including P1 tickets. The assigned handler supplies resolution details first. Tickets never close automatically.

In the extended application, the requester can reopen an unsatisfactory resolution. Reopening returns the ticket to Open and the manager's review queue, clears the assignment and preserves the ticket ID, previous resolution and history. The manager can assign an eligible handler again according to priority.

## Persistent conversations — extended application

Each ticket has one continuous conversation tied to its ticket ID. Conversation becomes available when an IT Team Member is assigned. The requester and current assignee can exchange messages, and authorized IT Managers can review the conversation. Manager-only P1 conversation behavior must be agreed before implementing chat.

Every message records its ID, ticket ID, actual sender ID, message content and creation timestamp. Messages remain available through resolution, reopening and reassignment. A new assignee reads the earlier conversation and continues it. Earlier messages retain their original authorship and timestamps.

Access is limited to the requester, currently assigned support member and authorized IT Managers. Reassignment removes the previous member's access through that assignment. Store chat in `ticket_messages`; store structured system activities separately in `ticket_events`.

## Optional images — extended application

Employees may attach supporting images. Ticket creation must work without images. Validate size and permitted image formats on the backend, with a maximum of 10 MB per image. The allowed formats and exact byte limit must be specified in the upload contract.

Store actual files in file/object storage and metadata in `ticket_attachments`. Metadata includes ID, ticket ID, uploader ID, file name, storage key, content type, file size and creation timestamp. Apply ticket access rules when serving images. Select the storage provider during the attachment increment.

## Data model

P6 has five core business tables:

| Table | Purpose and contract |
| --- | --- |
| `users` | Employee, manager and member identities, roles, account status and timestamps. A owns the canonical role/status field names. |
| `categories` | UUID, name, ticket type, optional description, active flag and creation/update timestamps. B owns the schema. |
| `category_agents` | Category-to-member eligibility. The existing contract uses composite key `(category_id, user_id)` and `joined_at`. |
| `tickets` | Ticket identity/reference, requester, type, category, custom concern, employee-selected priority, description, status, assignment, resolution, version and timestamps. C owns the final DTO and SQL field names. |
| `ticket_events` | Ticket ID, event type, actor/system, relevant event details and creation timestamp. |

Use primary and foreign keys, appropriate indexes, and transactions for related changes. Ticket category and ticket type must match. Preserve historical records and enforce category eligibility during assignment.

The supplied draft's alternative names such as `agent_id`, `assigned_to`, `is_active` for users and `resolution_details` are proposals. Agree on one shared contract before renaming existing fields. The categories table already has `updated_at`; update it when category data changes.

After P6, add `ticket_messages` and `ticket_attachments` with their own migrations. Once applied, migration files remain immutable; subsequent schema changes use new migrations.

## Technology and architecture

The backend uses Rust, Tokio, Axum, SQLx, PostgreSQL, Serde/JSON and tracing. Utoipa/OpenAPI documents the APIs. Docker Compose supplies local PostgreSQL. GitHub Actions runs agreed checks; `xh` and `jq` support demonstrations.

```text
HTTP request -> Axum routes -> application workflow and domain rules
             -> SQLx repository -> PostgreSQL -> JSON response
```

Keep pure validation and business rules independently testable. Handlers extract requests and return HTTP responses. Application functions coordinate transactions, repositories execute bound SQL, and infrastructure handles configuration, logs and later file storage.

## Scope and three-person ownership

The original three-day P6 milestone targets a working local API with users, categories/memberships, ticket creation, manager assignment, priority enforcement, the four-stage lifecycle, audit events, filtering/sorting/pagination, OpenAPI and automated tests.

| Member | Branch | Responsibilities |
| --- | --- | --- |
| A | `feat/support-users` | Users table and APIs, roles/account status, shared actor contract; later authentication and sessions. |
| B — Shreyas | `feat/support-categories` | PostgreSQL foundation, typed categories, Incident category restrictions, Others support, eligible-member mappings, migration coordination and category tests. |
| C | `feat/support-tickets` | Ticket/reference generation, employee-selected priority, description/custom-concern validation, assignment, lifecycle, audit events and ticket tests; later conversations, attachments and reopening/reassignment. |

Each owner supplies their service's validation, business rules, SQL, routes, tests and documentation. B and C coordinate Others recognition and assignment eligibility; A supplies consistent identity and role data. Agree IDs, field names, JSON errors, enum values and schema order before integration.

Later increments add authentication/access controls, persistent chat, image storage, reopening/reassignment, a frontend and role dashboards, notifications, SLA tracking, reporting and production deployment. Their acceptance tests belong to those increments.

## Implementation progress

- Foundation PR #6 was merged into `develop`. It provides local PostgreSQL configuration, the SQLx connection pool, JSON logging and `/health` and `/ready`.
- Shreyas manually verified database outage and recovery: `/health` returned 200, `/ready` returned 503 during the outage, then `/ready` returned 200 after PostgreSQL recovered.
- On B's branch, migration `0002_create_categories.sql` has been applied locally through SQLx. Two category validation unit tests passed for name trimming and blank-name rejection.
- Category insertion/query APIs, member mappings and the new Incident/Others rules still need implementation. The installed category migration does not yet enforce the new Hardware exclusion.
- Main currently carries planning documentation. Feature completion and integration must be confirmed through reviewed PRs and recorded test results.

## Acceptance checks

For P6, demonstrate:

- User creation, updates, role validation and activation/deactivation.
- Category create/read/update behavior, case-insensitive duplicates, Incident Hardware exclusion and availability of Others.
- Eligible-member mapping and rejection of invalid members.
- Both ticket types, mandatory nonblank descriptions up to 1,000 characters, and free-text custom concerns for Others.
- P1–P4 creation, P1 manager ownership, manager-only P2–P4 assignment, and rejection of ineligible members and self-claim attempts.
- Valid lifecycle transitions, resolution submission, requester confirmation and atomic event history.
- Filtering, deterministic sorting, pagination and concurrent-update handling.
- Formatting, linting, unit tests, real-PostgreSQL integration tests, OpenAPI and a repeatable local demonstration.

For later increments, verify authenticated access, message sender/timestamp preservation, conversation continuity after reassignment, old-assignee access removal, reopening, image size/type validation, optional upload behavior and authorized image retrieval.

## Agile delivery and Git workflow

Use small backlog items with an owner, acceptance criteria, implementation, tests and documentation. Review each increment with a teammate and record the demonstration result and follow-up work. Keep requirement, design, test and release documentation aligned with each accepted change.

```text
feature branch -> teammate-reviewed PR into develop
develop -> reviewed release PR into main
```

This retains the team's agreed integration branch. Use normal pushes. On each feature branch, fetch and merge updated `origin/develop` before integration, resolving conflicts with the responsible owner. Teammates run separate local database containers from the shared Compose file.

## Supporting documentation

These documents retain earlier detailed contracts and must be reconciled where this README changes priority ownership, description limits, categories or milestone scope:

- [Team assignments and interfaces](docs/ticketing/TEAM_TASKS.md)
- [Requirements and acceptance criteria](docs/ticketing/REQUIREMENTS.md)
- [Architecture and transactions](docs/ticketing/DESIGN.md)
- [Agile delivery and release workflow](docs/ticketing/DELIVERY.md)

The P1–P6 learning evidence spans Linux/CLI and Git, Rust types and ownership, error handling, Docker, generics, SQL/transactions, tests, CI and asynchronous HTTP APIs. Standalone course exercises are tracked separately from this application's deliverables.
