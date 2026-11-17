# Internal Company Support API

A planned Rust REST API for employees to raise and track **Infra** and **HR**
support requests. Categories organize the queues, eligible support agents take
responsibility, and employees confirm whether an issue is resolved.

**Status:** documentation and design stage. The ticketing application, database
migrations, API endpoints and tests have not been implemented. The final project
name is undecided; the repository is currently named AssetDesk.

## Workflow

```text
Admin prepares users, categories and agent memberships
  -> Employee raises a ticket in a category
  -> Eligible agent claims it and starts work
  -> Agent resolves it with an explanation
  -> Employee closes it or reopens it with a reason
```

Ticket states: `open -> in_progress -> resolved -> closed`.
Reopening a resolved ticket returns it to `open` without an assignee and retains
its earlier resolution in history. Tickets do not close automatically.

Examples include an Infra request for a faulty charger and an HR request for an
attendance correction. The workflow takes selected ideas from
[Jira Service Management's request classification](https://support.atlassian.com/jira-service-management-cloud/docs/what-are-request-types-in-itsm),
[ServiceNow's defined lifecycle](https://www.servicenow.com/docs/r/it-service-management/incident-management/c_IncidentManagementStateModel.html)
and [Zendesk's resolution/closure distinction](https://support.zendesk.com/hc/en-us/articles/8263915942938-About-the-ticket-lifecycle-and-ticket-statuses).
Our application uses flat categories, a fixed four-state workflow and manual
requester closure; the first milestone is defined below.

## First milestone

The proposed three-day local prototype covers:

- User creation, retrieval, listing, updates and account deactivation.
- Category management and category-agent membership.
- Ticket creation, details, category/status filters, stable sorting and pagination.
- Eligible-agent claiming with version checks.
- Validated lifecycle transitions and transactional event history.
- Unit tests, real-PostgreSQL integration tests, OpenAPI and an `xh`/`jq` demo.

Five business tables are planned: `users`, `categories`, `category_agents`,
`tickets` and `ticket_events`.

Comments, admin reassignment, advanced queries, notifications, attachments and
SLA automation remain follow-up work. P6 recommends a separate authentication
increment. The company-use target still requires tested login, session handling
and access rules: employees see their own tickets, agents see their category
queues, and admins oversee all queues. Synthetic actors in a local prototype do
not establish employee identity or protect HR information.

## Three-person ownership

| Member | Service | Created branch | First deliverable |
| --- | --- | --- | --- |
| A | Users and account lifecycle; authentication/access follow-up | `feat/support-users` | User contract, migration and create/get/list |
| **B: Shreyas** | **Categories, agent membership and shared foundation coordination** | **`feat/support-categories`** | **Small shared foundation PR; then category create/get/list** |
| C | Tickets, claiming, lifecycle and history | `feat/support-tickets` | Ticket types/rules, schema and create/get/list |

Each owner supplies validation, routes, business rules, SQL, tests, OpenAPI and
documentation for their service. A/B also help with integrated eligibility and
concurrency tests. The shared foundation and ordered schema must be reviewed
before dependent handlers are treated as complete.

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
