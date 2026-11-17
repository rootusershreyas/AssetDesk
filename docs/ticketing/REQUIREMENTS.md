# Internal support ticketing requirements

Proposed 2026-11-17 following the owner's scope change, CR-004. Nothing in this
document is a claim of implemented ticketing behavior or completed acceptance.
The project name remains undecided.

Current baseline: ticketing plans and PostgreSQL configuration only. The old
implementation has been removed from this working tree; its Git history remains.
There is no Rust manifest, application, migration set, test suite or CI workflow
yet. Existing PostgreSQL data/volume is preserved. Before new migrations run,
record a database compatibility decision, preferably a separate empty development
database for the new baseline.

The [short-notice profile](SHORT_NOTICE_DESIGN.md) selects the five-table local
prototype from this broader target. Its reference patterns and P1–P6 evidence
map do not mark deferred features or standalone course exercises complete.

## Purpose and boundaries

Give employees one place to request company support and see who is handling it.
Start with flat, administrator-managed **Infra** and **HR** categories. Adding
Finance or Facilities later should require data changes, not a new Rust enum or
deployment. A category represents a support queue, not an employee's department.

Examples:

- Infra: Wi-Fi unavailable, charger replacement or meeting-room TV issue.
- HR: attendance correction, leave-policy question or payslip enquiry.

HR tickets record requests and responses. They do not calculate payroll, change
attendance balances or implement statutory compliance.

## Roles and target access

| Operation | Employee | Support agent | Admin |
| --- | --- | --- | --- |
| Raise a ticket in an active category | Yes, for self | Yes, for self | Yes, for self |
| Read/comment on own ticket | Yes | Yes | Yes |
| Read another employee's ticket | No | Only within an assigned support category | Yes |
| Claim an unassigned ticket | No | Only within an assigned support category | Assign an eligible agent instead |
| Work/resolve a ticket | No | Only if assigned to that ticket | Yes |
| Assign/reassign a ticket | No | No | Yes, to an eligible agent |
| Close/reopen a resolved ticket | Own ticket | Own ticket | Yes |
| Change users, roles or category memberships | No | No | Yes |
| Manage categories | No | No | Yes |

An agent remains a requester when reporting their own issue. An Infra agent has
no staff access to HR tickets unless also a member of HR. Admin access to all
categories is an explicit rule of this initial design; a separate HR-admin
permission model would be a later requirement.

Target identity comes from a validated server-side authentication context.
The request must not select its own role or impersonate a requester through a
body field/header. Every read, list, count, comment and history endpoint applies
the same ticket scope. Return 404 for a ticket outside the caller's visibility;
return 403 for a forbidden operation on an otherwise visible resource.

## Functional stories and acceptance

All stories below start in **Backlog**. They are proposed targets, not completed
implementation.

| ID | Owner | Story and acceptance |
| --- | --- | --- |
| T-X00 | B coordinates; A/C review | Bootstrap a new Rust manifest/runtime, config, DB startup, errors, logging, CI and test setup; agree database compatibility before applying a new migration baseline. Review the foundation PR into develop before feature branches depend on it. Only plans and PostgreSQL configuration currently exist. |
| T-A01 | A | Admin manages users: UUID, name, normalized unique email, role, active/inactive status and timestamps; invalid fields rejected; duplicate email 409; user list supports pagination. Preserve users referenced by ticket history. |
| T-A02 | A | Login/logout and shared active-user context: password hashes, revocable/expiring sessions, inactive-user rejection, safe errors and no credentials in logs. Real access enforcement requires this increment. |
| T-A03 | A + B/C | Enforce the permission matrix on all routes, including list/count queries and comments/events; negative tests show another employee and an Infra-only agent cannot inspect an HR ticket. |
| T-B01 | B | Admin creates/reads/lists/updates categories: trimmed name 1..80 characters, description up to 1,000, unique name ignoring case. Duplicate name 409; category list is paginated. |
| T-B02 | B | Admin deactivates/reactivates categories. New tickets in an inactive category fail with 409; existing tickets remain readable and actionable. No hard deletion of referenced categories. |
| T-B03 | B | Admin adds/removes agents from categories. Only active users with agent role may be added. A user can support several categories; duplicate addition is idempotent. Removing membership with an assigned, non-closed ticket fails 409 until outstanding work is handled under the lifecycle rules. |
| T-C01 | C | Employee raises a ticket: UUID, category, subject 1..160 characters, description 1..8,000, priority low/medium/high (default medium), open status, no assignee, version 1 and timestamps. Invalid/inactive categories rejected. |
| T-C02 | C | Authorized ticket lists support q, category, status, priority, assignee, created/updated sort, limit 1..100 and nonnegative offset. Search is literal; sorting has a stable ID tie-breaker. Detail returns the same visible resource. |
| T-C03 | C | Eligible agent claims an open/unassigned ticket, or admin assigns/reassigns to an eligible agent. After visibility checks, stale versions, invalid assignment eligibility and simultaneous conflicting claims fail 409 without partial changes; invisible tickets return 404 first. |
| T-C04 | C | Apply lifecycle rules below; require a resolution explanation and reopening reason; store each successful change and its actor in event history atomically. |
| T-C05 | C | Authorized users add employee-visible comments of 1..4,000 characters before closure and list comments/events in deterministic, paginated order. Comment/event reads follow ticket visibility; comments on closed tickets fail 409. |
| T-X01 | All | Integrate services, document actual OpenAPI behavior, run real-PostgreSQL tests and demonstrate the workflow. Authentication-deferred demos must be labelled as local prototypes. |

User deactivation or any role change making an assignee ineligible must return
409 while that user holds any assigned, non-closed ticket. Under the single-role
model, both agent-to-employee and agent-to-admin changes remove agent
eligibility. An admin reassigns/unassigns open
work or reassigns in-progress work first. Resolved work must be closed by the
requester/admin, or reopened and reassigned, before removing its agent's
eligibility. Deactivation invalidates sessions. No silent ticket reassignment
or deletion of conversation/history is allowed.

## Ticket lifecycle

| From | To | Allowed actor and condition |
| --- | --- | --- |
| open | in_progress | Assigned agent or admin; an eligible assignee is required |
| in_progress | resolved | Assigned agent or admin; nonblank resolution explanation required |
| resolved | closed | Requester or admin |
| resolved | open | Requester or admin; nonblank reopening reason required; clear assignee so the category can claim it again |

Claiming/assigning does not itself start work: an open ticket may have an
assignee. Assignments can change on open/in-progress tickets; an in-progress
ticket cannot be unassigned until reassigned. Closed tickets are final in this
increment. Repeated or other transitions return 409. Nothing automatically
closes a ticket because a clock expires.

Comments are visible to the requester; private staff notes are deferred. Closed
tickets are read-only, including comments. Category, requester, subject,
description and priority are immutable after creation in this increment.
Corrections can be described in comments before closure; metadata editing and
category transfer are later features. Ticket deletion, bulk updates and file
uploads are deferred.

## API and quality requirements

- Serde validates JSON, bounds and unknown fields; malformed requests return
  400, validation failures 422, unavailable resources 404 and conflicts 409.
- Creates return 201 with Location; successful reads/updates return 200;
  successful idempotent membership writes may return 204. Unsupported content
  type is 415; excessive bodies are 413. Errors share one documented JSON shape.
- SQLx binds values; PostgreSQL enforces uniqueness, foreign keys and check
  constraints. Multi-row changes use transactions and rollback on failure.
- Tokio/Axum handle concurrent HTTP/DB work. Password hashing must not block
  an async worker when authentication is implemented.
- Rust enums model role/status/priority and Result models domain errors.
  Transition and validation functions form a pure core around an async SQL/HTTP
  shell. These are useful Rust applications, not features exclusive to Rust.
- Structured tracing records request IDs, status, latency and ticket IDs;
  exclude passwords, tokens, ticket descriptions, HR details and comment bodies.
- OpenAPI documents actual implemented routes, DTOs, errors and access rules.
  Integration tests use real PostgreSQL, including rollback and concurrency.

## Milestones and P6 interpretation

P6 calls its project-management domain an example and emphasizes practical API
design, relationships, eligibility, validation, concurrency and integration
testing. This proposal changes the business domain; it does not claim that a
ticket is literally a P6 project or that every original entity requirement is
unchanged. Confirm the alternative-domain mapping with the evaluator if needed.

P6's final instruction defers authentication/authorization, estimating around a
week. A three-day local prototype can prioritize T-A01, T-B01/02, T-C01/02/03/04
and a small end-to-end demonstration, reducing optional scope as needed.
T-A02/03 remain required for the requested company application. If deferred,
record the demo's identity mechanism and access limitations explicitly, use
synthetic data locally and do not claim employee or HR privacy enforcement.

No completed ticketing tests, business acceptance, production release or cost
savings have been demonstrated by creating this requirements document.
