# Company support ticketing API (name pending)

A planned Rust REST API for employees to raise and track internal support
requests, starting with **Infra** and **HR**. Administrators manage users and
categories; support agents work on tickets in their assigned categories.

The final name is undecided. The repository remains `AssetDesk`; the existing
local PostgreSQL setup uses `assetdesk`. No Rust package is currently present.

## Problem and workflow

An employee reports “My laptop charger is not working” under Infra, or
“My attendance record needs correcting” under HR. The request enters the
appropriate queue, an eligible agent takes responsibility, and the employee
tracks the conversation and outcome.

```text
Employee raises ticket -> Category queue -> Agent claims / admin assigns
-> In progress -> Resolved with explanation -> Employee closes or reopens
```

This is an internal service desk. Workflow references include Jira Service
Management (Atlassian), ServiceNow, Zendesk and the earlier ServiceDesk Plus
reference. The [reference and learning design](docs/ticketing/SHORT_NOTICE_DESIGN.md)
links the official sources and records the small patterns selected for this API.

## Current implementation status

**The ticketing system is a design proposal; its endpoints are not implemented.**
The old application source, Cargo files, tests, migrations, examples, CI workflow,
build output and superseded inventory documents have been removed from this
working tree. The folder now contains the ticketing plans, PostgreSQL development
setup, Git ignore rules and review/issue templates.

Git history and existing PostgreSQL data/volumes are preserved. Bootstrap a new
Rust foundation through a reviewed PR before implementing the planned services.
Choose a compatible migration baseline before connecting new migrations to an
existing database. There is no runnable API, generated OpenAPI or application
test suite in this cleaned baseline.

## Target features

| Area | Planned behavior |
| --- | --- |
| Users | Create, read, list, update and deactivate accounts; employee, agent and admin roles |
| Categories | Create/update/list/deactivate Infra, HR and future support categories |
| Support membership | Associate agents with one or more categories |
| Tickets | Raise, view, search, filter, sort and paginate requests |
| Ownership | Eligible agent claims a ticket; admin assigns/reassigns it |
| Progress | Validated status transitions, resolution explanation and reopening |
| Collaboration | Employee-visible comments and an ordered event history |
| Concurrency | Version checks and transactional updates prevent conflicting changes |
| Access | Employees see their own tickets; agents see their queues; admins oversee all |

HR ticket contents must not be visible to unrelated employees or Infra-only
agents. Lists, counts, detail views, comments and history follow the same access
rules. Roles in a users table alone do not enforce these rules.

## Three-person ownership

| Member | Service | Created branch | First deliverable |
| --- | --- | --- | --- |
| A | Users, login/session lifecycle and shared authorization | feat/support-users | User contract, CRUD and active/inactive accounts; agree authentication increment |
| **B: Shreyas** | **Categories, category-agent membership and shared database coordination** | **feat/support-categories** | **Shared foundation first; then category migration, CRUD and membership** |
| C | Tickets, claiming/assignment, lifecycle, comments and history | feat/support-tickets | Ticket migration and create/get/list contract |

Use modules in one Rust application and one PostgreSQL database. Each owner
supplies their API documentation and tests. Detailed stories and the proposed
branch split are in the [delivery plan](docs/ticketing/DELIVERY.md).

## P6 and the time limit

The proposed three-day prototype uses five business tables: users, categories,
category_agents, tickets and ticket_events. It prioritizes category queues,
eligible-agent claim, four lifecycle states, transactional history, basic
filter/sort/page operations and tests. Comments, admin reassignment, advanced
queries and login/access integration remain follow-up work. The full target
above is broader than this milestone.

The [P1–P6 learning map](docs/ticketing/SHORT_NOTICE_DESIGN.md#applying-all-six-parts-naturally)
connects each course part to planned implementation and demonstration evidence.
Standalone algorithm/data-structure exercises remain separate learning work.

Use Rust, Tokio, Axum, SQLx, PostgreSQL, Serde/JSON, `tracing` and Utoipa/OpenAPI;
verify APIs with `xh`, `jq` and integration tests against real PostgreSQL.

P6 says the project-management domain is an example. This domain exercises the
same API concepts: resource relationships, assignment eligibility, validation,
HTTP errors, collection queries and concurrency.

P6's closing instruction explicitly defers authentication/authorization and
estimates about a week for it. Our requested login/role enforcement is additional
scope. The proposed three-day milestone is a **local API prototype**; secure
employee use requires a separately verified authentication and access increment.
No three-day production-readiness commitment is made.

Email ingestion/notifications, attachments, SLA escalation, approval workflows,
AI features, a frontend and payroll processing are deferred.

## Documentation and workflow

- [Detailed A/B/C assignments and first tasks](docs/ticketing/TEAM_TASKS.md)
- [Created branches and recommended Git workflow](docs/ticketing/BRANCHING.md)
- [Short-notice scope, product references and P1–P6 learning map](docs/ticketing/SHORT_NOTICE_DESIGN.md)
- [Complete team implementation handoff](docs/ticketing/TEAM_HANDOFF.md)
- [Requirements, permissions and acceptance criteria](docs/ticketing/REQUIREMENTS.md)
- [Data model, API contracts and transaction rules](docs/ticketing/DESIGN.md)
- [A/B/C tasks, three-day proposal and Agile lifecycle](docs/ticketing/DELIVERY.md)
- [Documentation index](docs/README.md)
- [Ticketing decisions and cleanup record](docs/ticketing/CHANGES.md)

```text
feature/docs branch -> teammate-reviewed PR into develop -> release PR into main
```

Use normal pushes. Do not force-push, directly update protected branches or
claim incomplete work as delivered. The three feature branches were created
from origin/develop. The planning/cleanup baseline needs teammate review into
develop before members merge those plans into their branches. No main/develop
merge or repository-rule change is performed by creating the branches.

## Database setup only

Preserve an existing .env. The following commands start/check PostgreSQL;
application code will be added in the foundation increment.

```bash
test -f .env || cp .env.example .env
docker compose up -d --wait db
docker compose exec db psql -U assetdesk -d assetdesk -c 'SELECT current_database(), current_user;'
```

The host database port is 5434. The proposed API port is 3000 once an application
exists. Cargo build/run/test and API health calls become available after the
new foundation is implemented. Do not reset a database volume to make a future
migration mismatch disappear; decide the database baseline explicitly.
