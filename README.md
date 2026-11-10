# ReleaseReady

A Rust REST API for managing programmers, teams, projects, and tasks, with a release-readiness report that explains unfinished work and blockers.

**Status: planning only.** No application code, database schema, or runnable service has been created yet. This README is the starting agreement for a three-person, three-day P6 project.

## Project goal

Answer: **Is this project's recorded checklist complete, and what still needs attention?**

```text
Register programmers
    -> Create a team and designate its lead
    -> Create a project owned by the team
    -> Create and assign tasks
    -> Track work and handle conflicting updates
    -> Retrieve readiness and blocker information
```

Readiness reflects the recorded task checklist. It does not certify software quality, execute tests, or deploy software.

## Scope and technology

Implement all programmer, team, project, and task requirements from P6, then add one derived readiness endpoint.

| Technology | Purpose |
| --- | --- |
| Rust and Tokio | Domain models, error handling, async runtime |
| Axum | HTTP routing, extraction and responses |
| SQLx and PostgreSQL | Async queries, migrations, transactions and constraints |
| Serde | JSON request and response models |
| tracing and log | Structured logging and log interoperability |
| Utoipa / OpenAPI | Document implemented endpoints and errors |
| Rust integration tests | Verify HTTP behavior with real PostgreSQL |
| Docker Compose | Reproducible local PostgreSQL setup |
| GitHub Actions | Formatting, linting and tests |
| xh and jq | Manual API checks and evaluation demonstration |

Keep this release API-only. Authentication/authorization is deferred as instructed in P6. Bookings, AI integration, notifications, dashboards, payroll, and deployment automation are outside this iteration.

## Required data model

Use UUID identifiers and timestamp fields with time zones. Keep the original P6 field sets.

| Table | Fields |
| --- | --- |
| `programmers` | ID, name, email, active/inactive status, creation timestamp |
| `teams` | ID, name, description, lead ID, creation timestamp |
| `team_members` | Team ID and programmer ID as a composite primary key |
| `projects` | ID, name, description, owning team ID, status, creation timestamp |
| `tasks` | ID, project ID, title, description, status, priority, optional assignee ID, optional due date, creation/update timestamps, version |

Relationships:

- Programmers can belong to multiple teams.
- Every team has one lead who is also a member.
- A team owns multiple projects; a project belongs to exactly one team.
- A project contains multiple tasks; a task belongs to exactly one project.
- A task may be unassigned. An assignee must belong to the project's owning team.

Statuses:

```text
Programmer: active | inactive
Project:    planned | active | completed | archived
Task:       todo | in_progress | blocked | done
Priority:   low | medium | high
```

## Required API operations

All business endpoints use `/api/v1`.

| Resource | Operations |
| --- | --- |
| Programmers | Register, retrieve, list, update, deactivate, list assigned tasks |
| Teams | Create, retrieve/list, update, list/add/remove members, replace lead, list owned projects |
| Projects | Create, retrieve, list, update, assign/change owning team, change status, list tasks |
| Tasks | Create, retrieve, list, update, delete, assign/reassign/unassign, change status/priority, filter/sort/paginate |
| Readiness | `GET /api/v1/projects/{id}/readiness` |

Rules to enforce:

- Programmer email is unique without considering case.
- Removing the current lead requires assigning a replacement atomically.
- Membership and owning-team changes must not invalidate lead or task-assignment rules.
- Task updates include the current version; successful updates increment it and update the timestamp. Stale updates return `409 Conflict`.
- Optional task assignment and due date can be cleared explicitly.
- Collections have bounded pagination and deterministic sorting. Whitelist sort fields and bind SQL values.
- Agree and document any additional deactivation policy before coding.

## Readiness feature

Example response:

```json
{
  "project_id": "...",
  "project_status": "active",
  "ready": false,
  "total_tasks": 10,
  "completed_tasks": 7,
  "completion_percent": 70,
  "blocked_tasks": 2,
  "unfinished_high_priority_tasks": 1,
  "unassigned_unfinished_tasks": 1,
  "overdue_unfinished_tasks": 1,
  "reasons": [
    "Three tasks remain unfinished",
    "Two tasks are blocked",
    "One high-priority task is unfinished"
  ]
}
```

- `ready` is true only when at least one task exists and all tasks are `done`.
- Empty projects return `ready: false`, 0 percent completion, and an explanation.
- Overdue means a task is unfinished and its due date is before the reporting date. Use UTC dates consistently for this version.
- Completed tasks are excluded from unfinished-work warnings.
- Calculate readiness from current data; do not store a separate readiness flag.
- Keep project status independent: reporting readiness does not change it.
- Keep the calculation pure, passing task data and the reporting date as inputs.
- Read the report's task data consistently so concurrent changes do not produce contradictory counts.

## Three-person work division

Each contributor owns their feature's validation, SQL, HTTP errors, tests, and OpenAPI definitions. Coordinate shared files and migrations before editing them.

### A — Programmers, teams, application foundation

- [ ] Initialize Cargo, Tokio/Axum, configuration, shared errors, router, logging, and health endpoint.
- [ ] Implement programmer registration, retrieval, listing, updates, and deactivation.
- [ ] Implement team creation, retrieval/listing, and updates.
- [ ] Implement membership addition, removal, and listing.
- [ ] Implement lead replacement and transactional team/initial-membership creation.
- [ ] Test duplicate email, multiple memberships, invalid lead removal, valid replacement, and rollback.

### B — Shreyas: database coordination, projects, readiness

- [ ] Coordinate PostgreSQL/Compose, SQLx pool, shared application state, and migration order with A and C.
- [ ] Implement project creation, retrieval, listing, updates, team assignment, and status changes.
- [ ] Implement the team-project listing.
- [ ] Validate owning-team changes against existing task assignments.
- [ ] Implement the pure readiness calculation and readiness endpoint.
- [ ] Test missing/empty projects, incomplete/complete checklists, blocker counts, due dates, and project-team changes.
- [ ] Coordinate PostgreSQL integration-test configuration and CI.

### C — Tasks, assignment, query features, concurrency

- [ ] Implement task CRUD with every P6 field.
- [ ] Implement assignment, reassignment, and unassignment with team-membership validation.
- [ ] Implement project-task and programmer-task listings.
- [ ] Implement status/priority changes and optional due dates.
- [ ] Implement filtering, sorting, bounded pagination, and absent-versus-null PATCH behavior.
- [ ] Implement version checks and meaningful concurrent-update tests.
- [ ] Test valid/unassigned tasks, invalid assignments, optional-field clearing, collections, and stale versions.

## Three-day schedule

| Day | A | B | C |
| --- | --- | --- | --- |
| 1 | App setup and programmer/team basics | Database coordination and project CRUD | Task CRUD |
| 2 | Membership/lead rules and tests | Readiness report and project integrity | Assignment, collection queries, versions |
| 3 | Integration, documentation and fixes | Integration, CI and readiness demo | Concurrency checks, integration and fixes |

Agree the schema and API shapes at the start of day 1. Setup PRs land early; later features can use valid SQL fixtures while another member's endpoints are unfinished. Preserve day 3 for verification rather than adding features.

## Planned project structure

This is a proposed layout; these implementation files do not exist yet.

```text
src/
  main.rs
  lib.rs
  app.rs
  config.rs
  db.rs
  state.rs
  error.rs
  telemetry.rs
  openapi.rs
  features/
    programmers/
    teams/
    projects/
    tasks/
    readiness/
migrations/
tests/
```

Use small feature modules. Handlers manage HTTP; services coordinate rules/transactions; repository functions execute SQL. Readiness calculation stays independent of Axum and SQLx.

## Branch and review workflow

```text
feature branch -> reviewed PR -> develop
develop -> tested release PR -> main
```

- Create task branches from updated `develop` and target `develop` with feature PRs.
- Use normal pushes. Do not force-push or rewrite shared history.
- Keep PRs small; include tests and documentation with the behavior they implement.
- Another teammate reviews before merge. Suggested rotation: A reviews B, B reviews C, C reviews A.
- Keep `.env` and real employee/company data out of Git.
- A new GitHub repository will need its own rules configured; existing EnviroHub settings do not transfer automatically.

## Final acceptance and demonstration

- [ ] Every original P6 resource field and operation is implemented.
- [ ] Invalid lead removal and out-of-team assignment are rejected.
- [ ] Filtering, sorting, pagination, and stale task versions work.
- [ ] Readiness explains empty, incomplete, blocked, and complete checklists correctly.
- [ ] Errors use consistent JSON and HTTP status codes.
- [ ] Structured logs, OpenAPI, and real-PostgreSQL integration tests are complete.
- [ ] Formatting, Clippy, and tests pass; setup instructions work from a clean checkout.
- [ ] Demonstration uses xh/jq: create team/project/tasks, show not-ready, reject invalid operations, finish tasks, show ready.

The first implementation task is A's minimal Cargo/Axum foundation, while B and C agree the database schema and task contract in parallel.
