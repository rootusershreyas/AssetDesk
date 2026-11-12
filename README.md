# AssetDesk

An IT equipment handover and employee equipment-clearance REST API.

**Status: planning.** This README defines the scope and implementation plan for a three-person, three-day Rust API prototype. Application code and a runnable service have not been created yet. The repository is currently named ReleaseReady; the application is AssetDesk.

## 1. Goal and organizational use

Answer three questions:

1. Who currently holds each company device?
2. Can this device be issued to another employee?
3. What equipment must an employee return before their equipment clearance is complete?

Example: IT registers a laptop, issues it to an employee, records its return and condition, and retains the custody history. Before the employee leaves, IT retrieves a report of outstanding equipment. Asset availability and clearance come from assignment records.

Scope: a three-person, three-day API prototype. Useful for demonstrating and piloting equipment custody with sample data. It is not a full HR, payroll, procurement, or inventory accounting system. Authentication and authorization are deferred, following the final instruction in P6. Employee IDs identify borrowers; they do not authenticate the person calling the API.

P6 explicitly permits choosing a different project. Its programmers/teams/projects/tasks are the sample domain, not compulsory entities for AssetDesk. Preserve its API design, technology, validation, concurrency, documentation, and testing learning objectives.

## 2. Three-day scope

### Required

- Register, retrieve, list, and update employees; activate or deactivate their account for new equipment issues.
- Register, retrieve, list, and update individually tagged physical assets.
- Change an asset's operational status.
- Issue available equipment to an active employee.
- Return equipment and record its condition.
- Retain assignment history and retrieve individual assignments.
- Filter, sort, and paginate collections.
- Report an employee's outstanding equipment and equipment clearance.
- Derive overdue assignments from expected return times.
- Prevent concurrent duplicate assignments and stale returns.
- Provide JSON errors, structured logs, OpenAPI documentation, real-PostgreSQL integration tests, and xh/jq examples.

### Only after the required checks pass

- Revise or clear an outstanding assignment's expected return time using its current version.
- Asset summary counts or CSV export.

### Outside this iteration

Authentication, permissions, a frontend, employee approval workflows, email/Slack notifications, background jobs, reservations, purchasing, depreciation, payroll, attachments, barcode/QR scanning, device agents, and automated device locking or wiping.

No hard deletion endpoints are needed for the prototype. Deactivate employees and retire assets; preserve assignment history.

## 3. Workflow

```text
Register employee + register asset
    -> Issue available asset
    -> Record current custody
    -> Query employee equipment-clearance report
    -> Receive and inspect returned asset
    -> Record return and condition
    -> Retrieve updated clearance and retained assignment history
```

An expected return time is an administrative deadline. Passing it marks an outstanding assignment overdue; it never automatically returns, unassigns, reissues, or disables a physical device.

Employee deactivation and equipment clearance are separate. An employee can become inactive while still holding equipment. New issues are blocked, returns remain permitted, and equipment clearance remains incomplete until all items are returned. The report is about equipment only, not HR or financial clearance.

## 4. Technology and learning objectives

| Learning area / technology | Concrete use |
| --- | --- |
| Rust types and ownership | Typed request/domain models; borrowed inputs for validation; small employee/asset/assignment ID newtypes where useful |
| Enums, Option, Result | Operational states, conditions, optional expected returns, explicit business failures |
| Collections and functions | Lists of outstanding items and a pure equipment-clearance calculation |
| Tokio and async/await | Async HTTP handling and SQLx database I/O |
| Axum and HTTP | Resource routes, extraction, validation, consistent responses and status codes |
| Serde / JSON | Request/response serialization and field naming |
| SQLx / PostgreSQL | Pool, migrations, joins, constraints, transactions, indexes and custody history |
| Concurrency | Competing issues, issue versus deactivation/maintenance, versioned returns |
| tracing / log | Structured request/operation logs and library log interoperability |
| Utoipa / OpenAPI | Documented endpoints, DTOs, filters and error responses |
| Rust tests | Pure-function unit tests and HTTP integration tests against real PostgreSQL |
| Docker Compose | Reproducible local PostgreSQL with a health check |
| Git / GitHub Actions | Reviewed feature PRs and formatting, linting and test checks |
| Linux, xh, jq | Run services, inspect logs and reproduce manual API demonstrations |

This applies the earlier course units through normal development: P1 shell/Git, P2 Rust and information modeling, P3 error handling/memory/collections/Docker, P4 generic types and CI, P5 SQL/testing/pure functions, and P6 practical HTTP APIs. No need to force every earlier exercise into the product.

Rust helps express valid states and explicit failures. A reusable `Page<T>` response is a small, useful application of generic types. PostgreSQL transactions and constraints enforce exclusive custody across concurrent requests and multiple application instances; Rust ownership alone does not provide that database guarantee.

## 5. Database design

Use UUID primary keys and UTC `TIMESTAMPTZ` values. Generate identifiers in the application. Use text columns with database CHECK constraints for the small sets of allowed states, and matching Rust enums.

```text
employees 1 -------- many assignments many -------- 1 assets
```

### employees

| Field | Type / meaning |
| --- | --- |
| id | UUID primary key |
| name | Nonempty text, maximum 120 characters |
| email | Normalized email, unique ignoring case, maximum 254 characters |
| status | active or inactive; default active |
| created_at | Server-generated timestamp |
| updated_at | Server-generated timestamp |

Normalize and validate email consistently before writing; enforce uniqueness in PostgreSQL as well. Use syntactic validation, not a claim that the email address has been verified.

### assets

| Field | Type / meaning |
| --- | --- |
| id | UUID primary key |
| asset_tag | Unique company identifier; trim and uppercase, maximum 64 characters |
| name | Nonempty text, maximum 120 characters |
| category | Nonempty text, maximum 60 characters, such as laptop or monitor |
| operational_status | in_service, maintenance or retired; default in_service |
| created_at | Server-generated timestamp |
| updated_at | Server-generated timestamp |

The asset tag is immutable after registration. Do not store a second `assigned` or `available` flag. An asset is available when it is in service and has no outstanding assignment. Asset responses include this derived availability.

### assignments

| Field | Type / meaning |
| --- | --- |
| id | UUID primary key |
| employee_id | Required foreign key to employees |
| asset_id | Required foreign key to assets |
| issued_at | Server-generated timestamp |
| expected_return_at | Optional timestamp; future when creating an assignment |
| returned_at | Optional server-generated timestamp; NULL means outstanding |
| condition_out | good or fair |
| condition_in | NULL before return; good, fair or damaged after return |
| version | Positive integer, initially 1; increment on every supported assignment update |

Assignment employee/asset identifiers and issue details are immutable. Moving equipment to another employee means recording its return and issuing it again in a new assignment.

Enforce foreign keys, positive versions, expected return after issue, and return at or after issue. Constrain return timestamp and return condition so they are both absent or both present.

The important unique index is:

```sql
CREATE UNIQUE INDEX one_open_assignment_per_asset
ON assignments (asset_id)
WHERE returned_at IS NULL;
```

It permits historical assignments while enforcing at most one outstanding assignment per asset. PostgreSQL documents this conditional uniqueness mechanism in [Partial Indexes](https://www.postgresql.org/docs/current/indexes-partial.html).

Also index assignment employee IDs for employee history/report queries and asset IDs for full asset history. Unique email and asset-tag indexes serve their lookups. Add more indexes only for demonstrated query needs.

## 6. Business rules and concurrency

1. Issuing requires an existing active employee and an existing in-service asset with no outstanding assignment.
2. Maintenance or retired assets cannot be issued.
3. Changing an assigned asset into maintenance or retired is rejected until the return is recorded.
4. Employee deactivation prevents new issues but permits outstanding assignments and returns.
5. A return requires the current assignment version. Already returned assignments and stale versions return 409 Conflict.
6. Returning records the server's return time, return condition and incremented version together.
7. A damaged return places the asset into maintenance in the same transaction. Good/fair returns leave it in service and available.
8. Rejected transactions leave no partial updates.
9. Equipment clearance is derived; do not store an independently editable clearance flag.

Use a single lock order whenever more than one row is needed: **employee, then asset, then assignment**.

Issue transaction: lock the employee row, lock the asset row, recheck status and custody, insert the assignment, and commit. The partial unique index remains the final duplicate-custody safeguard. Employee status changes lock the employee; asset status changes lock the asset before checking custody. Consequently, issue requests serialize with those changes.

Return transaction: read the assignment's immutable employee/asset IDs, then lock employee, asset and assignment in that order. Recheck outstanding state/version, record the return and any maintenance change, and commit. Returns to inactive employees are valid.

Keep transactions short. Do not make network calls or wait for user input inside them. PostgreSQL explains row locks and consistent lock ordering in [Explicit Locking](https://www.postgresql.org/docs/current/explicit-locking.html).

If the optional deadline-edit feature is implemented, it must follow the same locking/version rules and reject returned assignments. Document missing-versus-null behavior: omission retains the deadline, explicit null clears it, and a timestamp replaces it.

## 7. API contract

All business routes start with `/api/v1`. Create operations return 201 with a Location header; successful reads and updates return 200 with JSON.

| Method | Route | Purpose |
| --- | --- | --- |
| POST | /employees | Register an active employee |
| GET | /employees | List employees; filter by status |
| GET | /employees/{id} | Get employee details |
| PATCH | /employees/{id} | Update name, email or active/inactive status |
| POST | /assets | Register a physical asset |
| GET | /assets | Search, filter, sort and paginate assets |
| GET | /assets/{id} | Get an asset and derived availability |
| PATCH | /assets/{id} | Update descriptive fields or operational status |
| POST | /assignments | Issue equipment |
| GET | /assignments | List current or historical assignments |
| GET | /assignments/{id} | Get one assignment |
| POST | /assignments/{id}/return | Record a physical return using the current version |
| GET | /employees/{id}/equipment-clearance | Get outstanding equipment and clearance |

Use assignment collection filters for employee history and asset history, rather than implementing extra duplicate listing endpoints:

```http
GET /api/v1/assignments?employee_id={id}&state=outstanding
GET /api/v1/assignments?asset_id={id}&state=all
GET /api/v1/assignments?overdue=true
GET /api/v1/assets?availability=available&category=laptop&sort=asset_tag&order=asc&limit=20&offset=0
```

Collection contract: `limit` defaults to 20 and must be 1..100; `offset` defaults to 0 and must be nonnegative. Allowlist sort fields per resource and append the ID as a stable tie-breaker. Reject invalid filters/sorts. Bind query values; never interpolate arbitrary client-provided SQL identifiers. Return `items`, `limit` and `offset`; total counts are optional.

Employees support status filtering and sorting by name or created_at; default created_at descending. Assets support category, operational_status and availability filters, a `q` substring search over name/tag, and sorting by asset_tag, name or created_at; default created_at descending. Availability values are available (in service with no outstanding assignment), assigned (an outstanding assignment exists), and unavailable (maintenance or retired). Assignments support sorting by issued_at; every collection accepts asc/desc ordering and uses the same direction for its ID tie-breaker.

Assignments default to all history, ordered by issued_at descending and ID descending. Supported assignment filters are employee_id, asset_id, state (all/outstanding/returned) and overdue. Overdue=true means outstanding with a non-null expected return before the captured UTC reporting time. Reject overdue=true combined with state=returned.

Example issue request:

```http
POST /api/v1/assignments
Content-Type: application/json
```

```json
{
  "employee_id": "11111111-1111-4111-8111-111111111111",
  "asset_id": "22222222-2222-4222-8222-222222222222",
  "condition_out": "good"
}
```

`expected_return_at` may be omitted/null or supplied as an ISO 8601 timestamp. Reject a supplied deadline at or before the server's issue time.

Example return request:

```http
POST /api/v1/assignments/{id}/return
Content-Type: application/json
```

```json
{
  "version": 1,
  "condition_in": "good"
}
```

For employee/asset PATCH requests, omitted fields remain unchanged; null for their non-nullable fields is invalid. Do not expose assignment ownership or return timestamps through generic PATCH. If the optional deadline-edit feature is added, use `PATCH /api/v1/assignments/{id}` with a required version and the explicitly defined expected_return_at update behavior.

Operational routes: `GET /health` checks process liveness; `GET /ready` checks database connectivity with a timeout; `GET /api-docs/openapi.json` serves the specification. Swagger UI is optional.

## 8. Equipment-clearance report

```json
{
  "employee_id": "11111111-1111-4111-8111-111111111111",
  "employee_status": "inactive",
  "equipment_cleared": false,
  "outstanding_count": 1,
  "overdue_count": 0,
  "outstanding_assets": [
    {
      "assignment_id": "33333333-3333-4333-8333-333333333333",
      "asset_id": "22222222-2222-4222-8222-222222222222",
      "asset_tag": "LAP-021",
      "name": "Development laptop",
      "expected_return_at": null
    }
  ]
}
```

Rules: existing employee plus zero outstanding assignments gives equipment_cleared=true, including an employee who has never borrowed anything. Missing employee gives 404. Returned assignments do not count as outstanding or overdue.

Keep the calculation pure: `calculate_clearance(outstanding_items, reporting_time)`. The HTTP/database layer fetches employee details and outstanding rows consistently in one SQL statement, then passes the resulting items and captured time into the calculation. Use a left join or equivalent so an existing employee with no outstanding assignments is retained and produces a clear report, rather than 404. Do not issue separate independently timed count and list queries.

The response describes the records at query time. It does not authenticate an operator, authorize an employee's departure, or perform any external HR action.

## 9. Errors and logging

Use one shared AppError and one JSON error format:

```json
{
  "error": {
    "code": "asset_already_assigned",
    "message": "This asset has an outstanding assignment."
  }
}
```

| HTTP status | Examples |
| --- | --- |
| 400 | Malformed JSON, malformed identifier or invalid query syntax |
| 404 | Employee, asset or assignment does not exist |
| 409 | Duplicate email/tag, asset already assigned, inactive employee on issue, unavailable asset, assigned asset status change, stale version or repeated return |
| 422 | Empty/oversized values, invalid condition/state, invalid return deadline or invalid field combination |
| 500 | Unexpected failure; generic client-facing message |
| 503 | Database unavailable or readiness check timed out |

Map known named constraints to business errors. Never expose SQL, connection URLs or raw database errors to clients.

Use structured request and operation logs with method, route/status, duration and relevant record IDs. Log successful issue/return outcomes after commit. Do not log request bodies, credentials, or employee email addresses by default.

## 10. Planned structure

These are planned files, not a generated scaffold:

```text
Cargo.toml
Cargo.lock
compose.yaml
.env.example
.gitignore
README.md
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
    employees/
    assets/
    assignments/
    reports/
migrations/
  <timestamp>_create_assetdesk_schema.sql
tests/
  common/
  employees.rs
  assets.rs
  assignments.rs
  clearance.rs
.github/workflows/ci.yml
```

Within a feature, separate HTTP DTOs/routes, domain/service rules, and SQL repository functions only as needed. Keep it small; no generic repository framework is required. Use a shared clonable AppState containing a PgPool. Pure report logic does not depend on Axum, SQLx or the current system clock.

Use a dedicated AssetDesk development database and an isolated test database setup. Keep .env ignored and commit an example containing development placeholders. Select and lock compatible crate versions together when creating Cargo.toml; commit Cargo.lock for the application.

## 11. Three-person ownership

Every contributor writes their own feature tests and OpenAPI definitions. Shared integration is collaborative, not one person's last-day cleanup.

### A: application foundation and employees

- Minimal Cargo/Tokio/Axum setup, configuration, shared errors, application/router shape, tracing and health route.
- Employee create/get/list/update and active/inactive behavior.
- Employee validation and duplicate-email handling.
- Employee tests, including return compatibility after deactivation and concurrent deactivation/issue with C.
- Coordinate the final router and OpenAPI registry; integrate exports supplied by B and C.
- Support C's HTTP/concurrency tests after employee endpoints work.

Deliverable: a runnable server and a tested employee module.

### B: Shreyas, database and assets

- PostgreSQL Compose service/health check, .env.example and database connection pool.
- Coordinate the single agreed initial migration for all three tables, constraints and indexes.
- Shared PgPool state and readiness endpoint with A.
- Asset create/get/list/update and operational-status rules with C.
- Asset search, filtering, deterministic sorting and bounded pagination.
- Asset tests and documentation.
- Coordinate isolated PostgreSQL integration-test configuration and CI; support C's database tests.

Deliverable: a reproducible database setup and tested asset module.

### C: custody transactions and clearance

- Assignment create/get/list, issue validation, row locking and conflict mapping.
- Versioned return, preserved custody history and damaged-return maintenance transaction.
- Assignment collection filters and employee/asset history through those filters.
- Pure equipment-clearance calculation and report endpoint.
- Tests for parallel issue, repeated/stale returns, rollback, deadlines and report transitions.
- Assignment/report OpenAPI definitions and demonstration requests.

Deliverable: a tested issue/return/history/clearance workflow.

Shared-file ownership: A edits Cargo.toml, app/router, error.rs and the OpenAPI registry; B edits db/state and the initial migration; each member owns their feature modules and tests. Coordinate overlapping changes in a small prerequisite PR. A and B review C's transaction behavior together.

## 12. Three-day execution

### Day 1: one working vertical flow

First 60-90 minutes together: freeze schema, statuses, JSON/error shapes, method signatures, lock order and test-database strategy.

| A | B | C |
| --- | --- | --- |
| Minimal server and employee endpoints | Database, initial migration/pool and asset endpoints | Assignment models/repository and issue operation using valid fixtures while other endpoints are in progress |

Merge small foundation/database PRs early. Day-1 gate: register an employee, register an asset, issue it, and read the assignment from PostgreSQL. If the team misses this gate, remove optional work immediately.

### Day 2: enforce correctness

| A | B | C |
| --- | --- | --- |
| Employee validation/status tests; shared HTTP/OpenAPI support | Asset status rules, collection queries, test setup/CI; support transaction tests | Return, history, clearance and concurrency tests |

Day-2 gate: a complete issue/return/clearance flow, rejected duplicate issues and invalid transitions, and working collection filters. Each feature already has its tests and documentation.

### Day 3: integration and demonstration

Integrate into develop, fix failures, complete OpenAPI/errors/logs, verify setup from a clean checkout, and rehearse the xh/jq demonstration. Freeze feature additions. Keep time for fixing real integration failures.

Required checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` against the isolated PostgreSQL test setup. CI runs the same checks with PostgreSQL available. Generate documentation from the implemented API, not an aspirational endpoint list.

## 13. Git workflow

```text
feature -> reviewed PR -> develop -> tested release PR -> main
```

Suggested AssetDesk feature branches, to create when implementation starts:

- A: feat/employees
- B: feat/assets-database
- C: feat/assignments-clearance

These are proposed implementation branch names. Create them from develop when implementation starts.

Create branches from updated develop, target develop with feature PRs, use normal pushes, and integrate shared changes through normal merges. Do not force-push. Another teammate reviews before merging. Proposed rotation: A reviews B, B reviews C, C reviews A; review transaction changes jointly.

Branch protections are repository settings and must be configured/verified separately. A written workflow does not itself enforce required reviews or block direct/force pushes.

## 14. Acceptance tests and evaluation demo

Essential tests:

- Valid create/get/update/list operations and syntactic/semantic validation errors.
- Duplicate normalized emails and asset tags return 409.
- Inactive employee, maintenance asset and retired asset cannot receive new issues.
- Two overlapping issue requests for one available asset yield one 201, one 409 and exactly one outstanding assignment in PostgreSQL.
- Concurrent issue versus deactivation or maintenance never commits a combination prohibited by the chosen rules. An employee may become inactive after a valid issue; no issue may succeed based on an already inactive employee.
- Assigned assets cannot be moved into maintenance/retired by a status update.
- Returns to inactive employees succeed; already returned and stale-version requests fail with 409.
- Damaged return and maintenance update commit together; failures roll back together.
- Return leaves history, releases current custody, and increments assignment version.
- Equipment-clearance report is false with outstanding items and true with none; missing employee is 404.
- Overdue rules use a supplied reporting time; no-deadline and returned items are handled correctly.
- Filters, invalid sort keys, pagination bounds and tie-break sorting work.

Use real PostgreSQL for database behavior, especially constraints and concurrency. Use separate connections and overlapping requests for race tests; sequential duplicate requests are not evidence of concurrent correctness. Test fixtures use sample employees and isolated data.

Demo story:

1. Register two employees and one available laptop.
2. Submit overlapping requests to issue that laptop to the two employees.
3. Show one success and one 409, then inspect the single outstanding assignment.
4. Show the winning borrower's equipment-clearance report as incomplete.
5. Deactivate that employee; show that custody remains visible and another issue to them is rejected.
6. Return the laptop using its current assignment version.
7. Show equipment clearance complete and the returned record still visible in history.
8. Show the passing real-PostgreSQL tests, OpenAPI specification and structured request logs.

For step 5's inactive-employee issue check, use a second available asset or return the first asset before the check so the failure specifically tests employee status rather than existing custody.

Evaluation pitch: "AssetDesk records company equipment custody, prevents conflicting issues, preserves handover history, and derives employee equipment clearance from outstanding assignments."

## 15. First implementation actions

1. Agree this schema, transaction policy and response contract as a team.
2. A creates the smallest runnable Cargo/Axum foundation.
3. B prepares the dedicated database, environment example and agreed baseline migration in coordination with A.
4. C prepares assignment/report domain models and pure report tests, then connects to the shared database contract.
5. Merge prerequisite setup early and complete the day-1 vertical flow before expanding features.
