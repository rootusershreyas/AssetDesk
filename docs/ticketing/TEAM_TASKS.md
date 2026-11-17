# A/B/C service handoff

Updated 2026-11-17 for manager-led IT support. This is the implementation
assignment, not evidence of completed services. The shared repository contains
planning documents. B has a local Rust scaffold and a reported successful
cargo check; this documentation update does not change that user-owned code or
claim that HTTP/database integration is complete.

Use [requirements](REQUIREMENTS.md) for scope and acceptance,
[design](DESIGN.md) for contracts and [delivery](DELIVERY.md) for Git/Agile work.

## Shared workflow and boundaries

An employee creates a Service Request or Incident. An IT Manager reviews it,
sets P1–P4 priority and assigns responsibility. P1 is handled personally by the
reviewing manager; P2–P4 goes to an eligible IT Team Member. The assigned handler
starts work and submits a resolution. The original requester confirms closure
or reopens with a reason. All changes retain history.

```text
Users (A) + Typed categories/membership (B)
  -> Create ticket -> Manager review and assignment (C)
  -> Start -> Resolve -> Requester confirms or reopens -> Retained history
```

The initial domain is IT support. HR requires a later, separately authorized
workflow; IT Manager visibility does not grant access to HR requests.

Keep one Rust application and PostgreSQL database, with feature-module ownership.
Each person supplies DTOs, validation, routes, service rules, bound SQL, OpenAPI,
tests and documentation. B coordinates shared files and migration order; A/C
remain responsible for their own service SQL.

| Member | Created branch | Business tables | First milestone |
| --- | --- | --- | --- |
| A | feat/support-users | users | User contract/CRUD, role/status rules and actor interface |
| **B: Shreyas** | **feat/support-categories** | **categories, category_agents** | **Foundation coordination, type-aware category CRUD and IT Member eligibility** |
| C | feat/support-tickets | tickets, ticket_events | Typed ticket/reference, manager review/assignment, lifecycle and history |

Authentication, comments, subcategories, reassignment/reprioritization,
advanced queries, notifications, dashboards and attachments remain later work.
Manager review/initial assignment and requester reopening/closure are core.

## Shared implementation contract

Agree before integrating handlers:

- Internal UUIDs, timestamptz timestamps and snake_case JSON fields.
- Roles employee/it_manager/it_member; account_status active/inactive.
- Types service_request/incident. UUIDs remain PK/FKs; immutable ticket_number
  uses SR/INC plus a per-type PostgreSQL sequence, such as SR1001 or INC1001.
- Priorities p1/p2/p3/p4. A new ticket has no final priority until manager review.
- States open/in_progress/resolved/closed. Resolved means Awaiting employee
  confirmation; closed means the original requester confirmed the outcome.
- One actor interface defined by A. B/C consume it rather than inventing
  independent identity/role headers. Synthetic local actors do not prove login.
- List default limit 20, maximum 100, offset 0 and deterministic ID tie-breaker.
- Common errors: 400 malformed, 422 validation, 404 missing/invisible resource,
  409 business/version conflict and safe 500/503 failures. Auth adds 401/403.
- Shared transaction-level PostgreSQL eligibility lock before row locks; see
  the [transaction contract](DESIGN.md#transactions-and-concurrency-contract).

The shared foundation supplies configuration, pool/migration startup, application
state, JSON errors, health/readiness, logging, OpenAPI composition, Page<T>,
fixtures and CI. B continues the local scaffold through a small reviewed PR;
this document does not reset or overwrite it. Before migrations, choose a separate
empty development database or an explicit compatibility path for retained data.

## A: users, roles and identity

A answers who may request support, manage IT work or handle assigned tickets.
B consumes active IT Member records; C consumes requester/manager/assignee
eligibility. The company target derives identity from validated sessions. The
local prototype may use documented synthetic actors only.

Initial story: T-A01. T-A02/03 remain the authentication/access follow-up.

users fields: id, name, normalized unique email, role, account_status,
created_at and updated_at. Preserve referenced accounts through deactivation.
Public responses exclude credential/session fields.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/users | IT Manager creates an account; 201 plus Location |
| GET | /api/v1/users | Manager-scoped paginated directory |
| GET | /api/v1/users/{id} | Permitted user details or 404 |
| PATCH | /api/v1/users/{id} | Manager updates metadata/role/account_status |

| Planned file | Responsibility |
| --- | --- |
| src/features/users/model.rs | User, Role and AccountStatus |
| src/features/users/dto.rs | Validated create/update/list inputs and safe responses |
| src/features/users/repository.rs | Bound SQL operations |
| src/features/users/service.rs | Normalization, uniqueness and role/status-change rules |
| src/features/users/routes.rs and mod.rs | HTTP responses, routing and OpenAPI |
| A-owned migration and tests/users.rs | Constraints and PostgreSQL evidence |

Suggested functions: validate_user_input, create_user, get_user, list_users,
update_user and current_actor/active-user contract.

Before changing a role/status, check whether it would invalidate any non-closed
assignment under the shared lock. An IT Manager is eligible for P1; an IT Member
is eligible for P2–P4 only with category membership. Promotion is not always safe:
an IT Member changed to manager can become ineligible for existing P2–P4 work.
The guard depends on C's ticket schema; do not silently skip it when absent.

First PR: user migration and create/get/list with normalization, duplicate email
409, invalid-input, missing-record and pagination tests. Then integrate guarded
updates/deactivation and supply synthetic employee/manager/member fixtures.

Authentication follow-up: first-manager setup, password lifecycle, login/logout,
session expiry/revocation, inactive-user rejection and backend scope tests.
User CRUD or a supplied demo actor does not complete that increment.

## B: categories and support membership — Shreyas

B answers what support is requested and which IT Members are eligible. C uses
B's typed-category and membership rules during creation and manager assignment.

Initial stories: T-X00 coordination and T-B01/02/03; T-B04 subcategories is later.

categories fields for the first milestone: id, ticket_type, name, description,
is_active, created_at and updated_at. Trim names, require 1..80 characters,
bound description at 1,000 and enforce case-insensitive uniqueness within each
ticket_type. The same category name may exist separately for requests/incidents.
Category ticket_type is immutable after creation.

category_agents uses composite key(category_id, user_id), foreign keys and
joined_at. Only active it_member accounts join this mapping. A reviewing P1
manager does not need category membership; C applies that separate priority rule.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/categories | Create typed category; 201 plus Location |
| GET | /api/v1/categories | Discover categories filtered by ticket_type |
| GET | /api/v1/categories/{id} | Permitted details or 404 |
| PATCH | /api/v1/categories/{id} | Update name/description/is_active |
| GET | /api/v1/categories/{id}/agents | List support membership |
| PUT | /api/v1/categories/{id}/agents/{user_id} | Manager adds eligible member idempotently; 204 |
| DELETE | /api/v1/categories/{id}/agents/{user_id} | Manager removes membership when allowed; 204 |

The /agents URI names the support mapping; the actual role value is it_member.
Category/membership writes are IT Manager operations in the authenticated target.

| Planned file | Responsibility |
| --- | --- |
| src/features/categories/model.rs | Category, TicketType contract and membership records |
| src/features/categories/dto.rs | Typed category inputs, filters and safe responses |
| src/features/categories/repository.rs | Category/membership SQL |
| src/features/categories/service.rs | Normalization, activation and membership eligibility |
| src/features/categories/routes.rs and mod.rs | HTTP behavior, route export and OpenAPI |
| B-owned migration and tests/categories.rs | Constraints and real-DB evidence |

Suggested functions: validate_category_input, create_category, get_category,
list_categories, update_category, add_member, remove_member and list_members.
Share is_eligible_member with C inside the caller's transaction. Do not open an
unrelated transaction or treat an earlier eligibility read as current.

Deactivation stops new tickets but preserves existing work. Removal with assigned,
non-closed work returns 409 under the shared lock. Membership creation depends
on A's users schema; removal guards depend on C's tickets schema. C owns matching
category.ticket_type to the ticket and checking activity during creation.

First PR: finish the shared foundation with A/C review. Then typed category
create/get/list, followed by PATCH and membership. Verify trimming/bounds,
case-insensitive duplicates within type, same-name cross-type categories,
invalid types, pagination, duplicate PUT, inactive/non-member-role rejection and
removal/configuration races against manager assignment.

Subcategory follow-up: add one-level parent_id within categories, require the
same ticket_type as its root and inherit root membership. Coordinate C's
subcategory reference/validation; do not add a sixth business table or expose
an unimplemented subcategory field in the first milestone.

## C: tickets, manager review and lifecycle

C combines A's users with B's categories. The inputs are ticket_type, category_id,
subject and description. New tickets are open, unassigned, unreviewed, version 1
and priority null. Manager review sets the authoritative priority/assignment.

Core stories are T-C01/02/03/04. T-C05 comments and T-C06 changes to reviewed
priority/assignment remain follow-up work.

Tickets retain UUID id, ticket_number, requester/category/type, nullable assignee,
subject/description, status, nullable priority, reviewed_by/reviewed_at,
resolution/root_cause, version and timestamps. Subject is 1..160 characters;
description is 1..8,000. Initial requester/type/category/content stays immutable.
Use per-type database sequences for unique references, never MAX()+1.

ticket_events stores actor, type, old/new values needed to explain the change,
explanation, resulting_version and time. One state/assignment mutation produces
one event/version. Review and assignment are one combined event, not two events
that collide with the ticket/version uniqueness rule.

| Method | Planned endpoint | Result |
| --- | --- | --- |
| POST | /api/v1/tickets | Create typed request/incident; 201 plus Location |
| GET | /api/v1/tickets | Scoped category/type/status/priority/reviewed filters, stable sort and pagination |
| GET | /api/v1/tickets/{id} | Visible details or 404 |
| POST | /api/v1/tickets/{id}/review | Manager sets priority and assignment atomically |
| PATCH | /api/v1/tickets/{id}/status | Start, resolve, confirm closure or reopen |
| GET | /api/v1/tickets/{id}/events | Scoped, deterministic history |
| GET | /api/v1/me/work-history | Own historical-work metadata without unrestricted ticket access |

| Planned file | Responsibility |
| --- | --- |
| src/features/tickets/model.rs | TicketType/Priority/Status and ticket/event records |
| src/features/tickets/dto.rs | Create/list/review/status inputs and safe responses |
| src/features/tickets/repository.rs | Scoped queries, locked mutations and event persistence |
| src/features/tickets/service.rs | Pure transitions plus actor/priority/assignment/version rules |
| src/features/tickets/routes.rs and mod.rs | HTTP behavior, route export and OpenAPI |
| C-owned migration and tests/tickets.rs | References, constraints, history and races |

Suggested functions: create_ticket, get_ticket_scoped, list_tickets_scoped,
review_and_assign_ticket, validate_transition, change_status and list_events.
Use one shared TicketType definition agreed with B, not independent mismatched
copies. Review input includes expected_version and priority; P2–P4 also requires
an eligible assignee. For P1, derive the acting manager as assignee and reject a
conflicting supplied ID. Manager review keeps status open and increments version.
Pending review means open with no assignee or review stamps; a reopened ticket
may retain a priority. The manager queue uses reviewed=false, not priority=null.

```text
open (manager review) -> open (reviewed/assigned) -> in_progress
 -> resolved (awaiting requester confirmation) -> closed
                            |
                            +-> open (reopen; manager review again)
```

Only the assigned handler starts/resolves. Resolution is required for all types;
incident resolution additionally needs root_cause. Only the original requester
confirms/closes or reopens their resolved ticket, including P1. The handling
manager gets no P1 closure shortcut unless also the original requester.

PATCH status input uses status and expected_version, resolution/root_cause when
resolving and explanation when reopening. Trim nonblank values and enforce
resolution 1..8,000 characters, incident root_cause 1..4,000 and reopening
explanation 1..4,000.

Reopening clears assignee, review stamps and current resolution/root cause,
retains the last priority/history and returns to pending manager review. Closed
tickets reject mutation. Reassignment/reprioritization is later work and must
revalidate the priority/assignee combination atomically.

Current IT Members see assigned tickets and their own requests. Former handlers
receive only their own historical-work projection from events; assignment history
alone does not grant access to the ticket's current details/conversation. Managers
see all IT tickets; employees see their own. Apply scopes before SQL pagination.

First PR: pure rules/DTOs and ticket/event schema after A/B migrations, then
create/get/list. Add review/assignment and lifecycle in small PRs. Test reference
uniqueness/type prefix, category/type mismatch, pending defaults, P1-manager and
P2–P4-member eligibility, stale versions, competing manager reviews (one success,
one 409, one event), requester-only closure, incident root cause, reopen resets,
rollback and history visibility. No self-claim endpoint belongs to this milestone.

## Integration and acceptance

Merge the reviewed planning baseline into develop, then the foundation and
ordered schema: users -> typed categories/membership -> tickets/events. A/B
configuration guards become complete only after C's schema and real race tests.
A/B help C with fixtures, concurrency tests and the end-to-end demonstration.
Sync each branch by normally merging updated origin/develop; see DELIVERY.md.

A slice is Done when agreed criteria work, tests pass for that commit, OpenAPI
and run notes match, another teammate reviews, and unfinished scope stays visible.
Demo a Service Request and Incident, manager P1/P2 assignment paths, resolution,
requester confirmation/reopening and retained history. Document synthetic local
identity explicitly; company use requires the separate authentication/access
increment. No changed ticketing implementation or passed application tests are
claimed by this documentation update.
