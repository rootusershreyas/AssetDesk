# A/B/C service handoff

Prepared 2026-11-17. This is the implementation assignment for the selected
five-table local prototype. Current project files contain Markdown plans only;
application source, configuration, Docker/PostgreSQL setup, migrations, tests
and CI still need to be built.

Use [DELIVERY.md](DELIVERY.md) for branch setup and synchronization, and
[REQUIREMENTS.md](REQUIREMENTS.md#first-milestone-and-follow-up-scope) for milestone boundaries.
The branches below are ownership assignments; their creation is a separate Git
operation and is not evidence of implemented services.

## Shared product context

An employee raises an Infra or HR support request. It enters that category's
queue. An active agent belonging to that category claims it, starts work and
resolves it with an explanation. The requester closes it or reopens it with a
reason. Every state/assignment change retains a history event.

```text
Users (A) + Categories/membership (B)
  -> Ticket and eligible-agent claim (C)
  -> Start -> Resolve -> Close or reopen -> Retained history
```

One Rust application and one PostgreSQL database, with separate feature modules.
Each person owns their module's DTOs, validation, routes, business rules, SQL,
OpenAPI annotations, tests and documentation. B coordinates shared files and
database order; coordination does not make B responsible for everyone else's SQL.

| Member | Branch assignment | Business tables | Initial stories |
| --- | --- | --- | --- |
| A | feat/support-users | users | T-A01; identity contract for later T-A02/03 |
| **B: Shreyas** | **feat/support-categories** | **categories, category_agents** | **T-X00 coordination, T-B01/02/03** |
| C | feat/support-tickets | tickets, ticket_events | T-C01, basic T-C02, claim slice of T-C03, T-C04 |

Login/sessions, enforced company access, comments, admin assignment/reassignment,
advanced queries, notifications and attachments remain follow-up work. Record
partial T-C02/T-C03 delivery without marking their full parent stories complete.

## Before implementation: agree the shared contract

The first foundation PR must establish the new Rust manifest, application state,
configuration, DB pool/migration startup, JSON errors, health/readiness, logging,
OpenAPI composition, generic Page<T>, fixture conventions and CI checks.
B coordinates it; A/C review the contracts before importing them.

Agree these values together:

- UUID identifiers; timestamptz timestamps; snake_case JSON fields.
- Roles employee/agent/admin; account_status active/inactive.
- Ticket states open/in_progress/resolved/closed; priority low/medium/high.
- Actor context supplies user ID and role. A defines the interface; B/C consume
  it rather than inventing independent identity headers or role checks.
- List defaults: limit 20, maximum 100, offset 0; stable ID sorting tie-breaker.
- Common errors: 400 malformed, 422 validation, 404 missing/invisible resource,
  409 business/version conflict and safe 500/503 failures. Auth adds 401/403.
- One transaction-level PostgreSQL eligibility-lock helper, acquired before row
  locks. All services follow the [transaction contract](DESIGN.md#transactions-and-concurrency-contract).

Existing PostgreSQL data/volume remains intact. Select a separate empty
development database or document a compatible migration approach before applying
new migrations. An existing database may contain old SQLx migration history.

## A: users and identity contract

### Purpose and handoff

A answers: who can raise a request, and which accounts are active support agents?
B uses A's users for category membership. C uses them for requester/assignee
eligibility. Account roles are domain metadata in the prototype; real identity
and permission enforcement require the later authentication increment.

Input: user name, email, role and permitted account updates.
Output: a user response with UUID, normalized email, role, account_status and
timestamps; no credential/session fields in the public response.

### Data and APIs to implement

users fields: id, name, email, role, account_status, created_at, updated_at.
Enforce normalized unique email, nonblank validated names, valid enum values and
preservation of referenced users. Do not hard-delete ticket participants.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/users | Create account; 201 plus Location |
| GET | /api/v1/users | Paginated users |
| GET | /api/v1/users/{id} | User details or 404 |
| PATCH | /api/v1/users/{id} | Update metadata/role/account_status; 200 |

No public signup or client-selected privileged role is part of the authenticated
company design. Those routes are admin operations when guards are integrated.

### Files and function responsibilities

| Planned file | Responsibility |
| --- | --- |
| src/features/users/model.rs | User, Role and AccountStatus types |
| src/features/users/dto.rs | Create/update/list input validation and safe UserResponse |
| src/features/users/repository.rs | Bound SQL create/get/list/update operations |
| src/features/users/service.rs | Normalization, duplicate mapping, role/status-change rules |
| src/features/users/routes.rs and mod.rs | HTTP extraction/responses, route export and OpenAPI |
| A-owned migration and tests/users.rs | Schema/constraints and real-database evidence |

Suggested functions: validate_user_input, create_user, get_user, list_users,
update_user and the shared actor/active-user lookup contract.

Any change removing agent eligibility, including agent -> admin under the
single-role enum, must return 409 while assigned non-closed tickets exist.
The check and update use the shared eligibility lock in one transaction.
Session revocation becomes part of deactivation when authentication is added.

This guard depends on C's tickets table. Land the ordered shared schema before
declaring role/status-changing operations complete; never silently skip the
guard when the table is absent.

### First slice and acceptance

Start with the users migration, pure normalization and create/get/list. Add
metadata updates and the guarded status/role mutations after schema integration.
The first small PR should include real-PG tests and matching OpenAPI/examples.

Required cases: valid creation, email normalization, duplicate email 409, invalid
input, missing user, pagination, inactive account persistence and blocked
eligibility-removing changes while assigned work exists. A also supplies
synthetic requester/admin/agent fixtures for B/C tests.

Later T-A02/03: login/logout, password setup/hashing, session expiry/revocation,
admin bootstrap and cross-user/category permission tests. Those stories are not
completed by CRUD or by accepting a synthetic actor in a local demo.

## B: categories and support membership — Shreyas

### Purpose and handoff

B answers: what kinds of support can employees request, and who may handle each
category? A category is a queue such as Infra/HR, not the requester's department.
C consumes active-category and eligible-agent queries when creating/claiming
tickets. New categories are data changes, not Rust enum changes.

Input: category name/description/is_active and a user ID for membership.
Output: normalized category responses and eligible support membership records.

### Data and APIs to implement

categories fields: id, name, description, is_active, created_at, updated_at.
Names are trimmed, 1..80 characters and unique ignoring case. Description is
bounded at 1,000 characters. category_agents uses composite key(category_id,
user_id), foreign keys to categories/users and joined_at.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/categories | Create category; 201 plus Location |
| GET | /api/v1/categories | Paginated category discovery |
| GET | /api/v1/categories/{id} | Category details or 404 |
| PATCH | /api/v1/categories/{id} | Update name/description/is_active |
| GET | /api/v1/categories/{id}/agents | List support membership |
| PUT | /api/v1/categories/{id}/agents/{user_id} | Add eligible agent idempotently; 204 |
| DELETE | /api/v1/categories/{id}/agents/{user_id} | Remove membership when allowed; 204 |

### Files and function responsibilities

| Planned file | Responsibility |
| --- | --- |
| src/features/categories/model.rs | Category and category-agent records |
| src/features/categories/dto.rs | Category/membership inputs, filters and responses |
| src/features/categories/repository.rs | Category SQL and membership operations |
| src/features/categories/service.rs | Normalization, activation and eligibility rules |
| src/features/categories/routes.rs and mod.rs | HTTP behavior, route export and OpenAPI |
| B-owned migration and tests/categories.rs | Constraints and real-database evidence |

Suggested functions: validate_category_input, create_category, get_category,
list_categories, update_category, add_agent, remove_agent and list_agents.
Agree is_eligible_agent with C: it must run inside the caller's transaction
rather than starting a separate transaction or trusting a previously read flag.

Only active users with role agent can join support categories; one agent may
join several. Duplicate PUT is idempotent. Category deactivation blocks new
tickets but preserves existing work/history. Removing membership with assigned,
non-closed work returns 409 under the shared eligibility lock.

Membership creation depends on A's users table. Membership-removal guards depend
on C's tickets table. Integrate those migrations before advertising complete
removal behavior. C owns checking category activity during ticket creation.

### First slice and acceptance

Your first implementation task is the small shared foundation PR, reviewed by
A/C. Then implement category create/get/list, followed by PATCH and membership.

Required cases: valid category, trim/case handling, duplicate name 409, invalid
lengths, missing category, activation changes, pagination, duplicate membership
PUT, inactive/non-agent membership rejection and removal blocked by open work.
Work with C to prove category deactivation and membership removal cannot race
ticket creation/claim into invalid state.

B coordinates migration names/order and shared app/router/error/OpenAPI changes.
Keep database setup/run notes current; do not rewrite A/C migrations independently.

## C: ticket workflow and history

### Purpose and handoff

C turns A's users and B's queues into a complete request lifecycle. The service
validates requester/category eligibility, shows the correct queue, records one
responsible agent and preserves what happened over time.

Input: category_id, subject, description and optional priority; then claim/status
actions with expected_version and resolution/reopen explanations.
Output: ticket details/pages and ordered event history.

### Data and APIs to implement

tickets fields: id, requester_id, category_id, nullable assignee_id, subject,
description, status, priority, positive version, created_at and updated_at.
New tickets are open/unassigned/version 1; priority defaults to medium.
Subject is 1..160 characters; description is 1..8,000. Initial metadata is
immutable in this milestone. The actor context supplies requester identity.

ticket_events records id, ticket_id, actor_id, event_type, applicable old/new
state/assignee, explanation, resulting_version and created_at. State-change
versions are unique per ticket; event history is append-only in application APIs.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/tickets | Create request; 201 plus Location |
| GET | /api/v1/tickets | Category/status filtering, stable created_at sort and pagination |
| GET | /api/v1/tickets/{id} | Visible ticket details or 404 |
| POST | /api/v1/tickets/{id}/claim | Eligible agent claims open/unassigned ticket |
| PATCH | /api/v1/tickets/{id}/status | Apply a permitted lifecycle transition |
| GET | /api/v1/tickets/{id}/events | Paginated, deterministic history |

### Files and function responsibilities

| Planned file | Responsibility |
| --- | --- |
| src/features/tickets/model.rs | TicketStatus, Priority, Ticket and TicketEvent |
| src/features/tickets/dto.rs | Create/list/claim/status inputs and response types |
| src/features/tickets/repository.rs | Scoped queries, locked mutations and event inserts |
| src/features/tickets/service.rs | Pure transitions plus actor/version/eligibility orchestration |
| src/features/tickets/routes.rs and mod.rs | HTTP behavior, route export and OpenAPI |
| C-owned migration and tests/tickets.rs | Ticket/history constraints, integration and concurrency cases |

Suggested functions: create_ticket, get_ticket_scoped, list_tickets_scoped,
claim_ticket, validate_transition, change_status and list_events.

```text
open -> in_progress -> resolved -> closed
                          |
                          +-> open (reopen with reason; clear assignee)
```

Claim increments version but keeps status open. Starting requires an eligible
assignee. Resolving requires an explanation; closing/reopening requires the
requester or permitted admin action. Closed tickets reject further mutation.

For every claim/state change: acquire the shared eligibility lock, lock ticket,
recheck actor/eligibility/state/version, update exactly once, insert event and
commit together. A stale/competing change returns 409 without a successful
event. In the authenticated target, an invisible ticket returns 404 before
revealing business-rule details. Apply visibility predicates in SQL before
filtering, pagination or any future counts.

### First slice and acceptance

Start pure transition functions and DTOs while A/B bootstrap their schemas.
Land the ticket/history migration after users/categories/membership, then
create/get/list, claim and lifecycle in small reviewable slices.

Required cases: active category/requester checks, invalid DTOs, correct default
state/version, stable pagination, invalid category membership, one successful
concurrent claim, stale-version conflict, allowed/forbidden transitions,
resolution/reopen explanations and rollback if event persistence fails.

Admin assignment/reassignment, comments, free-text search, advanced sorts and
company authentication are later slices. A/B help C with fixtures, concurrency
tests and the end-to-end demo because C owns the largest business-rule surface.

## Integration order and parallel work

| Checkpoint | Required dependency | What can proceed in parallel |
| --- | --- | --- |
| Planning baseline | Reviewed plans/setup into develop | All read contracts and refine their first story |
| Shared foundation | New Rust/runtime/config/error/DB/test/CI PR reviewed into develop | A/B/C draft their pure validation, DTOs and test cases |
| Ordered schemas | users -> categories/membership -> tickets/events | Module implementation against agreed field contracts |
| Eligibility integration | All five tables plus shared lock/helper available | A status/role guard, B removal guard and C claim tests |
| End-to-end acceptance | Working routes, errors, OpenAPI and real-PG checks | A/B fixtures/tests/docs; C closes workflow defects |

After each shared prerequisite is merged, fetch and merge origin/develop into
the member's branch using normal pushes. Do not copy files from another branch,
force-push shared history or mark missing dependencies as passed. Detailed Git
commands and merge policy belong in [DELIVERY.md](DELIVERY.md).

## Definition of Done and team demonstration

A story/slice is Done when its agreed criteria work, meaningful tests pass on
the exact commit, OpenAPI/run notes match, another teammate reviews the PR and
deferred scope remains visible. Use the company's actual Agile cadence and
record progress, blockers, demo feedback and a retrospective improvement.

Shared demo: create users -> Infra/HR categories -> agent memberships -> create
Infra ticket -> demonstrate one winning claim -> start -> resolve -> reopen or
close -> inspect history. Include invalid membership, stale version and rollback
evidence. Document the synthetic identity mechanism before a local prototype
demo; do not present it as tested company authentication or HR confidentiality.

Related context: [requirements](REQUIREMENTS.md), [design](DESIGN.md),
[delivery and branching](DELIVERY.md).
