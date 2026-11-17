# Short-notice design and P1–P6 learning map

Date: 2026-11-17. Status: proposed implementation baseline; ticketing code is
not built. Final name remains pending. This document selects a small prototype
from the broader [requirements](REQUIREMENTS.md); deferred items remain backlog.

Current files are Markdown documentation only. Code, manifests, Compose and
environment configuration, migrations, tests and CI are absent; Git history
remains. Existing database containers/data/volumes outside the repository are
unaffected. Bootstrap the new Rust/configuration foundation and record database
compatibility before any new migrations run.

## Product statement

Build a Rust HTTP API for a small company's employees to raise support requests
under Infra and HR. A category determines the support queue; an eligible agent
claims responsibility; the employee sees the outcome and can close or reopen a
resolved request. The team can explain, test and demonstrate the whole path.

Initial examples: Infra handles a charger/Wi-Fi problem; HR handles an attendance
or payslip enquiry. Categories are database records so another support function
can be added later without changing a Rust enum.

## Reference patterns and our decisions

These sources inform individual patterns. Our schema, names, endpoints and
workflow below are our own implementation choices.

| Reference | Relevant documented pattern | Small adaptation for this application |
| --- | --- | --- |
| [Jira Service Management, by Atlassian: request types](https://support.atlassian.com/jira-service-management-cloud/docs/what-are-request-types-in-itsm) | Classify requests and organize the agent's queue | One flat category per ticket, initially Infra/HR. A queue is a filtered ticket query, not a new queue engine/table. |
| [ServiceNow: incident lifecycle](https://www.servicenow.com/docs/r/it-service-management/incident-management/c_IncidentManagementStateModel.html) | Use defined states to track investigation, resolution and closure | Four fixed states with validated transitions and recorded explanations. |
| [Zendesk: ticket lifecycle](https://support.zendesk.com/hc/en-us/articles/8263915942938-About-the-ticket-lifecycle-and-ticket-statuses) | A solution can be offered before final closure; requests may need further work | Keep resolved separate from closed; requester explicitly reopens a resolved ticket with a reason. |

Atlassian is the vendor behind Jira; it is not a fourth independent workflow
reference. Product taxonomies differ: our Infra/HR categories are support queues,
not a claim to implement each vendor's request-type hierarchy.

Our employee/admin closes manually. Comments do not automatically reopen a
ticket. No timer automatically closes work. Vendor automation, custom workflow
editors, SLA engines and integration platforms are outside this prototype.

## Selected prototype scope

The target is a local, synthetic-data learning prototype for three developers
over roughly three days, subject to new foundation work and actual capacity.

| Include first | Detail / existing story |
| --- | --- |
| Shared working foundation | Config, DB/migrations, errors, health/readiness, logging, CI and test setup; T-X00 |
| Users | Create/read/list/update/deactivate, role metadata and active/inactive accounts; T-A01 |
| Categories | Create/read/list/update/deactivate; case-insensitive unique name; T-B01/02 |
| Category membership | Add/list/remove active eligible support agents; T-B03 |
| Tickets | Create/get/list; category/status filters; created_at ascending/descending plus ID tie-breaker; bounded pagination; T-C01 and the basic T-C02 slice |
| Claim | An eligible agent claims an open, unassigned ticket with expected_version; claim slice of T-C03 |
| Lifecycle and history | Start, resolve with explanation, close or reopen with reason; state/event transaction; T-C04 |
| Evidence | Unit and real-PG tests, matching OpenAPI, xh/jq demo, reviewed PRs and run documentation; T-X01 |

Use five business tables: users, categories, category_agents, tickets and
ticket_events. Comments introduce a sixth business table in a later slice;
authentication adds sessions when implemented.

Follow-up backlog: admin assignment/reassignment (rest of T-C03), advanced
search/filter/sort (rest of T-C02), comments (T-C05), login/session lifecycle
(T-A02), integrated access enforcement (T-A03), notifications, attachments,
reporting, category transfers, SLA reminders and configurable workflows.

Do not mark all of T-C02/T-C03 Done when only the selected slices are delivered.
Record accepted child tasks or partial scope and keep remaining acceptance open.
The broader permission and company-use requirements have not been removed.

## Core workflow and rules

```text
Admin prepares users -> Categories -> Agent memberships
Employee raises request -> Category's open queue
Eligible agent claims -> Assigned agent starts work
Agent resolves with explanation -> Requester closes or reopens with reason
Reopening -> Same category, open and unassigned
```

Fixed lifecycle: open -> in_progress -> resolved -> closed, with resolved ->
open for reopening. Claim leaves status open and increments the version.
Ticket creation is version 1; each successful state/assignment mutation advances
it once and appends an event. Requester/category/initial metadata stay immutable
in this increment. Closed tickets are read-only.

Important invariants:

- New tickets require an active category and active requester.
- An assignee must be an active agent belonging to the ticket's category.
- One ticket has at most one assignee; a competing/stale claim returns 409.
- Starting/resolving requires the assigned agent or permitted admin action.
- Resolving/reopening requires a nonblank explanation.
- Membership removal or any role/status change making an assignee ineligible
  is blocked while they hold non-closed assigned tickets. This includes changing
  agent to admin under the initial single-role model.
- Deactivating a category stops new requests while existing work continues.
- State changes and events commit together or roll back together.

For this prototype, offboarding waits until assigned work is closed or reopened
and handled. Admin reassignment/offboarding improvements are a follow-up because
they need more policy and tests. Never silently discard the outstanding work.

## Applying all six parts naturally

The attached files are learning references. Their exercises and environment
preferences are not an instruction to change the workstation or add unrelated
features. The evidence below is planned; there is no current application/test
suite or completed ticketing evidence.

| Learning part | Application in this project | Evidence the team should demonstrate |
| --- | --- | --- |
| P1: Linux, environment, processes, text tools, Git | .env configuration, Compose commands, diagnosing ports/processes, xh piped to jq, inspecting safe structured logs, normal feature/PR workflow | Reproducible run guide; a health/readiness call; explained process/port diagnosis and sanitized command output |
| P2: information model, Rust basics, Git | Stable UUID identity, changing ticket values plus dated events; structs/enums/pattern matching; small reviewed branches | Explain identity/state/time on one reopened ticket; show event history and a reviewed PR |
| P3: ownership, borrowing, collections, errors, debugging, Docker | Borrow validation input, own validated values, Vec for result pages, Option for assignee, Result for domain failures; PostgreSQL Compose | Explain a function's ownership choices; show errors and a reproduce -> hypothesis -> test -> fix record; run the containerized DB |
| P4: generics, complexity, CI/CD and Actions | Shared Page<T>; bounded SQL lists, stable sorting and useful indexes; fmt/Clippy/real-DB checks in CI; reviewed release process | Generic page DTO used by several lists; representative EXPLAIN evidence; exact-candidate CI and release/packaging records |
| P5: relational SQL, testing, functional core | FK/unique/check constraints, joins, transactions; pure normalization/transitions surrounded by async SQL/HTTP | Given-When-Then unit tests; real-PG eligibility, rollback and concurrent-claim tests |
| P6: HTTP APIs, async, logging and API tooling | Axum/Tokio/SQLx/Serde, status/Location semantics, filtering/sorting/pagination, tracing and Utoipa | OpenAPI matching real handlers; xh/jq lifecycle demo; safe logs and real PostgreSQL integration tests |

Generic response shape to implement in the shared API module:

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub offset: u32,
}
```

Add the appropriate serialization/OpenAPI derives when integrating it. The same
shape can hold UserResponse, CategoryResponse or TicketResponse. This snippet
is a design example, not a committed working module.

Keep transition rules as a pure function: current state plus requested action
and explanation yields the next state or a domain error. The service separately
checks actor eligibility/version and performs SQL inside a transaction. This
keeps the P5 functional core visible without introducing a framework.

For algorithm analysis, explain why the API bounds result size and lets the
database filter/order before LIMIT. Capture EXPLAIN for a representative
category/status queue query and inspect its chosen plan. Do not assert that an
index always wins on a small dataset or that all string searches become fast.

The linked-list, generic merge-sort/hash-table, quicksort, BST, Rustlings and
Exercism exercises remain separate learning-track evidence. Their completion
is not implied by this application. Use appropriate standard collections and
database ordering in the API. Optional Nix/editor/shell exploration can remain
in the existing development environment/run notes.

## Architecture and transaction scope

One Rust application, one PostgreSQL database; feature modules for users,
categories and tickets. Bootstrap and review a new shared foundation. Keep route
extraction, DTO validation, pure rules, service orchestration and SQL repositories
distinct enough that a teammate can explain the path of a request.

Begin with the [shared eligibility-lock contract](DESIGN.md#transactions-and-concurrency-contract).
One PostgreSQL transaction-level advisory lock serializes writes that affect
eligibility; acquire it before row locks. Ticket updates also verify the expected
version and insert an event in the same transaction. Centralize the helper.
This deliberately limits concurrent writes in the small prototype; it is not
evidence of high-throughput production performance.

Runtime logs should contain request/ticket IDs, status and latency, with no
passwords, tokens, descriptions or HR details. Verify the middleware once built
before claiming all fields exist. OpenAPI must list implemented routes, including
documented errors, rather than publishing planned operations as available.

## A/B/C execution sequence

| Owner | Main implementation | Shared support |
| --- | --- | --- |
| A | User DTOs/repository/routes, status/role changes and actor interface | Shared fixtures; help C test lifecycle and invalid users; estimate auth separately |
| B: Shreyas | Foundation coordination, category CRUD, membership and migration ordering | Share eligibility transaction helper; help C test races; run/setup documentation and container packaging |
| C | Ticket create/get/list, claim, pure lifecycle rules and event persistence | Integrate A/B contracts; document working ticket routes |

C owns the most complex rules; A/B should actively share fixtures, acceptance
examples and concurrency/integration tests instead of leaving all integration
work until C finishes.

Suggested milestone sequence:

1. **Foundation checkpoint:** bootstrap the Rust manifest/runtime, configuration,
   DB/error/logging, tests and CI, then review this shared base into develop.
   Only Markdown plans remain; Compose/environment setup must also be created.
   Record a separate empty development
   database or reviewed compatibility approach before new migrations run; retain
   existing PostgreSQL data/volume.
2. **Day 1:** agree schema/DTO/error/actor/transaction contracts; A users,
   B categories, C pure transitions and ticket schema. Merge ordered migrations.
3. **Day 2:** B membership; C creation/list/claim and state/events; A/B help with
   fixture and eligibility tests. Demonstrate create -> claim -> start -> resolve.
4. **Day 3:** finish reopen/close and failure cases, real concurrent tests,
   OpenAPI, xh/jq demo, run notes and teammate review. Package the app in a small
   Dockerfile after the core passes; add Compose configuration for PostgreSQL.
5. If capacity falls short, report unfinished criteria and replan. Do not replace
   real concurrency/rollback checks with extra endpoints or cosmetic work.

The three-day plan is a capacity proposal, not a guarantee. Budget the foundation
checkpoint explicitly rather than assuming it has already been reviewed.

## Authentication and company use

The owner requested user/access management. That remains part of the target.
P6's closing course guidance defers authentication/authorization and estimates
about a week for correct implementation. Accordingly, the prototype milestone
and the authenticated company-use increment are recorded separately.

Prototype user roles and synthetic actors demonstrate business rules, not
proof of employee identity or confidentiality. Use synthetic data locally;
an explicitly opt-in, loopback-only demo actor adapter, if implemented, remains
a demo mechanism. It must be rejected in normal authenticated mode.

Company-use acceptance requires actual sessions, admin bootstrap, inactive-user
handling and tested scope filtering. Test cross-user/cross-category access to
details/lists and any future comments/history/counts. Use the full access matrix
in REQUIREMENTS.md. Do not count T-A02/03 as Done before those checks pass.

## Small, reviewable acceptance demo

Use synthetic employee, two Infra agents, one HR agent and an admin:

1. Create Infra/HR categories and agent memberships; reject a duplicate category.
2. Raise an Infra ticket and show its open/unassigned version-1 response.
3. Attempt an HR-agent claim; reject ineligible assignment under the documented
   demo policy. In the authenticated target, a ticket outside visibility returns
   404 before revealing assignment eligibility details.
4. Run two eligible claims concurrently: one winner, one 409, one claim event.
5. Start and resolve the ticket, including explanation and version changes.
6. Reopen it with a reason; show the original resolution in history and no assignee.
7. Claim/start/resolve again, then close it and reject further mutation.
8. Show a stale-version failure, inactive-category failure and transactional rollback.
9. Show actual OpenAPI and the exact candidate's format/Clippy/real-PG checks.

For the local prototype, describe the identity mechanism before the demo and
avoid presenting a simulated actor as authenticated. A later company-use demo
adds real login/logout and forbidden-access cases.

## Agile and evidence

Use the company's actual Agile cadence. Select the above story slices in
planning; track Ready/In progress/Review-Test/Done; demonstrate working behavior;
record feedback and one retrospective action. Keep acceptance/tests/design/run
documentation in the same PR as the change.

Normal feature pushes -> teammate-reviewed PR into develop -> reviewed release
PR into main. No force pushes or automatic production deployment are introduced
by this design. Record checks against their exact commit, unfinished scope and
the business acceptance reviewer/operator before an organizational release.

References: [team handoff](TEAM_HANDOFF.md), [requirements](REQUIREMENTS.md),
[design](DESIGN.md), [delivery records](DELIVERY.md). No ticketing routes, tests,
ceremonies or release approvals are completed by this document. Created service
branches and synchronization are recorded in [BRANCHING.md](BRANCHING.md).
