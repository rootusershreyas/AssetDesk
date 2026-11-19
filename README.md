# IT Support Ticket Management API

A planned Rust REST API for employee **Service Requests** and **Incidents**.
IT Managers review and assign tickets; IT Team Members handle assigned work;
the original requester confirms whether the resolution is accepted.

**Status:** the shared Axum/PostgreSQL foundation and typed-category API are
implemented on `feat/support-categories`. User accounts, category membership,
tickets/events, PostgreSQL integration tests and OpenAPI remain in progress.
The final product name is undecided; the repository is currently named
AssetDesk.

## Workflow

```text
Manager prepares users, typed categories and member mappings
  -> Employee raises a Service Request or Incident
  -> System supplies UUID plus SR1001 / INC1001 reference
  -> IT Manager reviews priority and assigns responsibility
       P1: acting IT Manager handles it personally
       P2/P3/P4: eligible IT Team Member handles it
  -> Assigned handler starts work and records resolution
  -> Awaiting employee confirmation
  -> Original requester closes or reopens with a reason
```

Internal states remain `open -> in_progress -> resolved -> closed`.
`resolved` displays as **Awaiting employee confirmation**. P1 follows the same
requester-confirmation rule; the manager has no direct-closure shortcut.
Reopening returns the ticket unassigned to manager review and preserves the
earlier resolution/assignment in history. Tickets never close automatically.

Examples: a new charger is a Service Request; a Wi-Fi outage is an Incident.
Initial scope is IT infrastructure. HR remains a future separately authorized
workflow, with no automatic IT Manager access to HR records.
The workflow takes selected ideas from
[Jira Service Management's request classification](https://support.atlassian.com/jira-service-management-cloud/docs/what-are-request-types-in-itsm),
[ServiceNow's defined lifecycle](https://www.servicenow.com/docs/r/it-service-management/incident-management/c_IncidentManagementStateModel.html)
and [Zendesk's resolution/closure distinction](https://support.zendesk.com/hc/en-us/articles/8263915942938-About-the-ticket-lifecycle-and-ticket-statuses).
Our implementation decisions and milestone scope are defined below.

## Workflow changes

| Area | Earlier plan | Revised agreement |
| --- | --- | --- |
| Roles | Employee / agent / admin | Employee / IT Team Member / IT Manager |
| Classification | Flat support category | Service Request or Incident, then a type-aware category |
| Priority | low / medium / high at creation | P1/P2/P3/P4 decided by manager review |
| Ownership | Eligible agent self-claims | Manager assigns; P1 manager-owned, P2–P4 member-owned |
| Identity | UUID only | UUID plus immutable SR/INC reference |
| Resolution | resolved then requester closure | Same four states; resolved explicitly means awaiting requester confirmation for every priority |
| Member visibility | Category queue | Assigned tickets, own requests and limited own work history |

## First milestone

The proposed three-day local prototype covers:

- User creation, retrieval, listing, updates and account deactivation.
- Type-aware flat categories and category-member mappings.
- Service Request/Incident creation, SR/INC references and scoped ticket lists.
- Manager review, P1–P4 assignment rules and optimistic version checks.
- Resolution/root-cause records, requester confirmation/reopening and atomic events.
- Basic filtering, stable sorting, pagination and personal work-history metadata.
- Unit tests, real-PostgreSQL integration tests, OpenAPI and an `xh`/`jq` demo.

Five business tables are planned: `users`, `categories`, `category_agents`,
`tickets` and `ticket_events`.

## Implemented category contract

The category branch supplies the shared `TicketType` enum with
`service_request` and `incident`, PostgreSQL migrations for typed categories,
and these routes:

| Method | Route | Behavior |
| --- | --- | --- |
| `POST` | `/api/v1/categories` | Create a typed category and return `201` plus `Location` |
| `GET` | `/api/v1/categories` | List categories with optional `ticket_type`, `limit` and `offset` |
| `GET` | `/api/v1/categories/{id}` | Retrieve a category by UUID |
| `PATCH` | `/api/v1/categories/{id}` | Manager-only target: update its name, nullable description or active state |

Category names are trimmed, limited to 80 characters and globally unique
without case across both ticket types. Descriptions are limited to 1,000
characters. Hardware categories belong under Service Request, and category type
is immutable. Renaming is restricted to IT Managers once A's shared actor/auth
contract is integrated. C should reuse `features::categories::model::TicketType`
for tickets and reject missing, inactive or type-mismatched categories during
ticket creation.

The `category_agents` mapping waits for A's reviewed `users` migration. Its
membership endpoints and P2-P4 eligibility checks are therefore not part of
the current category PR.

Subcategories, comments, reassignment/reprioritization, advanced queries,
notifications, attachments, SLA automation and dashboards remain follow-up work.
P6 recommends a separate authentication increment. Company use still requires
tested login/sessions and backend access rules: employees see own requests,
members see assigned work, and managers oversee IT tickets. Previous involvement
does not grant ongoing full-ticket access. Synthetic actors do not prove identity.

## Three-person ownership

| Member | Service | Created branch | First deliverable |
| --- | --- | --- | --- |
| A | Employee/IT Manager/IT Team Member accounts; authentication/access follow-up | `feat/support-users` | User/actor contract, migration and create/get/list |
| **B: Shreyas** | **Typed categories, member mappings and shared foundation coordination** | **`feat/support-categories`** | **Continue the local foundation; then typed category create/get/list** |
| C | SR/INC references, manager review/assignment, lifecycle and history | `feat/support-tickets` | Ticket types/rules, schema and create/get/list |

Each owner supplies validation, routes, business rules, SQL, tests, OpenAPI and
documentation for their service. A/B also help with integrated eligibility and
concurrency tests. Keep the local Rust work intact; review the shared foundation
and ordered schema before treating dependent handlers as complete. The expanded
workflow needs replanning within the tentative three-day prototype capacity.

## Planned stack and learning evidence

One Rust application, with feature modules and one PostgreSQL database:
**Tokio, Axum, SQLx, Serde/JSON, tracing and Utoipa/OpenAPI**.

| Learning | Planned application |
| --- | --- |
| P1: Linux and CLI | Environment/process diagnosis, Git and `xh -> jq` pipelines |
| P2: Rust and information model | Structs/enums, stable IDs, changing state and dated events |
| P3: ownership, errors and containers | Borrowing, `Vec`, `Option`, `Result`, systematic debugging and Docker |
| P4: generics, complexity and CI/CD | `Page<T>`, bounded/indexed queries, GitHub Actions and reviewed delivery |
| P5: SQL, testing and functional core | Constraints/joins/transactions, pure rules and real-DB tests |
| P6: HTTP APIs | Async handlers, HTTP semantics, validation, concurrency, logs and OpenAPI |

Standalone course exercises remain separate learning evidence. This project
does not mark linked-list, sorting or hash-table assignments complete.

## Team workflow

```text
feature branch -> teammate-reviewed PR into develop
develop -> reviewed release PR into main
```

Use normal pushes and small PRs. Merge updated `origin/develop` into feature
branches without rewriting shared history. Repository protection and required
checks must be verified/configured separately; this README does not claim they
are enforced. The current branch baseline and synchronization steps are in the
[delivery and branching guide](docs/ticketing/DELIVERY.md).

## Documentation

- [A/B/C assignments, files, APIs and acceptance tests](docs/ticketing/TEAM_TASKS.md)
- [Milestone scope, requirements and target permissions](docs/ticketing/REQUIREMENTS.md)
- [Architecture, data model and transactions](docs/ticketing/DESIGN.md)
- [Agile delivery, branching, verification and release](docs/ticketing/DELIVERY.md)
