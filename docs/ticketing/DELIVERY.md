# A/B/C delivery and Agile records

Planning baseline: 2026-11-17. Product name pending. All ticketing stories begin
in Backlog. The working tree contains Markdown documentation only; application
code/manifests/Compose/environment setup/migrations/tests/CI are absent. This document
proposes work and records no completed planning meeting, review or release.

Use the [short-notice profile](SHORT_NOTICE_DESIGN.md) for the selected first
milestone and P1–P6 demonstration map. The full service ownership below includes
follow-up work; comments, admin reassignment, advanced queries and authentication
are outside that initial five-table prototype.

## Ownership and created branches

| Member | Service | Created implementation branch |
| --- | --- | --- |
| A | Users, authentication context and authorization integration | feat/support-users |
| **B: Shreyas** | **Categories, agent membership and shared DB coordination** | **feat/support-categories** |
| C | Tickets, assignment, status, comments and events | feat/support-tickets |

The branches were created locally and on origin from the same origin/develop
commit, b56c317. Their initial contents are that documentation baseline, without
application code. Review the documentation baseline PR into develop, then normally
merge updated origin/develop into each branch. T-X00 remains a real prerequisite:
B coordinates a newly bootstrapped Rust/runtime/config/DB/test/CI foundation PR and A/C
review it before feature branches depend on it. Existing
PostgreSQL data/volume is retained; agree a separate empty development database
or a reviewed compatibility approach before running new migrations.

For a self-contained briefing, see the [complete team handoff](TEAM_HANDOFF.md).
The [detailed assignments](TEAM_TASKS.md) specify each person's APIs/files/tests;
the [branching guide](BRANCHING.md) gives checkout, sync and review instructions.

One application, one database, three module owners. Each person implements
their own DTOs, routes, services, SQL, OpenAPI annotations and tests. Shared
files need coordination rather than concurrent independent rewrites.

## A: users and identity

1. T-A01: user migration, role/status enums, normalization and CRUD/list DTOs.
2. Define the user lookup/eligibility contract needed by B and C; preserve
   referenced accounts and coordinate role/status changes removing eligibility.
3. Agree the authenticated-context/guard contract before B/C wire target routes.
4. T-A02: implement login/logout, hashing, session expiry/revocation and
   inactive-user behavior in a separately estimated increment.
5. T-A03 with B/C: enforce and test all permissions, including scoped reads.

First PR: T-A01 with real-database tests, OpenAPI and an explicit note if
authentication is deferred. No authentication implementation currently exists
in this working tree.

## B: your categories and support membership service

1. Agree users FK/table names and migration order with A; agree membership and
   assignment eligibility rules with C. Record database compatibility before the
   new baseline is applied; preserve existing PostgreSQL data/volume.
2. T-B01: category migration/model, create/update validation and repository.
   Create/get/list/PATCH routes; normalize names and map duplicate constraints
   to 409. Search/paginate consistently with the shared contract.
3. T-B02: active/inactive behavior; existing tickets remain accessible while new
   creation is blocked. C implements the ticket-create side of that contract.
4. T-B03: membership migration and idempotent PUT/DELETE/list routes. Reject
   inactive/non-agent users; coordinate shared eligibility locking with A/C.
5. Add unit tests for normalization/bounds and PostgreSQL tests for duplicate
   names, persistence, idempotency and assignment-removal restrictions. Add
   access-negative cases once A's guards are integrated.
6. Document implemented routes/examples and hand off the exact membership
   query/transaction helper to C. Update acceptance evidence in your PR.

First coding slice: **T-B01 category create/get/list**, using A's agreed user
contract only when adding membership. Avoid mixing ticket handlers or login
implementation into your categories PR.

## C: ticket workflow

1. T-C01/02: ticket migration, create/get/list, validation and filters; depend on
   A users and B categories. Category/requester identity is immutable initially.
2. T-C03: claim and admin assignment/reassignment with versions, eligibility
   checks, shared locking and transactional event history.
3. T-C04: pure lifecycle validation, status endpoint and resolution/reopen rules.
4. T-C05: comments/events and deterministic paginated reads; all target reads
   apply ticket visibility, including nested endpoints.
5. Real-PG tests cover concurrent claim, stale versions, forbidden eligibility,
   rollback, filtering and transition errors. Coordinate scoped-access tests
   with A and membership/deactivation races with B.

First PR: create/get/list and valid category handling. Follow with small claim/
assignment and lifecycle PRs rather than a single unreviewable implementation.

## Proposed three-day prototype milestone

This is an initial capacity proposal, not a delivery promise or a three-day
enterprise replacement. Budget the new shared foundation before feature work;
no executable scaffold currently exists in this working tree.

| Day | A | B | C | Integrated checkpoint |
| --- | --- | --- | --- | --- |
| 1 | User contract, migration and basic CRUD | Category model/create/get/list | Agree ticket DTOs, pure rules and migration | Reviewed data/API contracts; migrations apply cleanly |
| 2 | User update/deactivation and tests | Category update/deactivation; membership if ready | Ticket create/get/list; claim and version checks | Synthetic requester -> category queue -> successful single claim |
| 3 | Fix integration; document identity limitations; help C test | Test/handoff B; resolve schema conflicts; help C test races | Core transitions/event history | Real-DB tests, OpenAPI, xh/jq demo and teammate review |

At planning, select stories to fit actual capacity. Comments, advanced queries
and admin reassignment remain follow-up work. Preserve core validation,
migration integrity and concurrency checks; record incomplete story slices.

P6 explicitly defers authentication/authorization; user CRUD can be demonstrated
without asserting access enforcement. If the prototype follows that deferral,
document how synthetic actors are supplied and run it locally with synthetic
data. Arbitrary actor IDs are not authentication. Company employee/HR use is
blocked until T-A02/03 and negative access tests pass. Do not quietly drop this
requested capability or mark role names as equivalent to secure permissions.

## Agile lifecycle and records

Use the company's existing Agile framework and cadence; these are not yet
specified. The three-day proposal is a milestone, not evidence of a Scrum sprint.

| Activity | Current artifact | Required record/evidence |
| --- | --- | --- |
| Discovery and requirements | README and REQUIREMENTS.md | Business reviewer, examples, scope decisions |
| Refinement/planning | Story IDs and owner split above | Selected stories, acceptance, dependencies and capacity |
| Design | DESIGN.md | Reviewed API/schema/transaction decisions |
| Implementation | Feature PRs | Code, migrations, matching OpenAPI and unit/PG tests |
| Integration and review | PR into develop | Exact-commit CI, different teammate's review and working demo |
| Feedback and improvement | Iteration record below | User feedback, retrospective action and owner |
| Release | Checklist below | Version/commit, reviewer/operator, verification and recovery record |
| Maintenance | Change/incident record | Reproduction, impact, fix/test PR and follow-up |

Definition of Ready: a story has an owner, observable acceptance criteria,
agreed dependencies/contracts and a realistic slice.

Definition of Done: implemented behavior matches acceptance/OpenAPI, meaningful
checks pass for that commit, a teammate reviews it, documentation/evidence is
updated and deferred scope remains visible. A local prototype is not Done for
authenticated-company-use stories.

```text
Iteration/milestone, dates and goal:
Selected story IDs, owner, reviewer and dependencies:
Daily progress, blocker and next action:
PR/commit, check commands/results and acceptance evidence:
Demo feedback and business reviewer:
Incomplete stories and reason for replanning:
Retrospective action, owner and follow-up:
```

## Verification and release gates

Each owner supplies tests for their rules; integrated tests must prove:

- Duplicate category names and invalid inputs yield the documented errors.
- Inactive categories reject new tickets while existing ones can progress.
- Nonmembers/inactive users cannot become ticket assignees.
- Two overlapping claims produce one success, one 409 and one claim event.
- Stale state changes fail, and event-write failures roll back state changes.
- Membership removal/deactivation cannot race an assignment into invalid state.
- Target permissions hide other employees' and unrelated categories' HR tickets,
  including through list/count, comments and history.

After bootstrapping the Rust application and test/CI setup, run formatting,
Clippy and tests against real PostgreSQL; check actual OpenAPI and an xh/jq
create -> claim -> resolve -> close demo. Those commands cannot run from this
planning-only baseline. No ticketing test has been run by writing these plans.

- [ ] Exact release candidate passes its required checks and teammate review.
- [ ] Target authentication/authorization tested before company use.
- [ ] Runtime docs/configuration/migrations match the candidate.
- [ ] Operator, environment, HTTPS/secrets and backup/restore procedure recorded.
- [ ] Migration/recovery rehearsed on a separate disposable database.
- [ ] Business acceptance and known limitations recorded.

Feature/docs PRs target develop; release PRs target main. Use normal pushes and
required review; never force-push or rewrite history to hide past activity.
Publishing changes, repository protection and a release remain separate actions.
