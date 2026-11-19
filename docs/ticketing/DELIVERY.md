# Delivery, Agile and Git workflow

Updated 2026-11-17 for manager-led IT support. The shared baseline is planning
only; B has a local Rust scaffold with a reported successful cargo check.
That local work is not overwritten or treated as a completed HTTP/DB foundation.
No ticketing API, acceptance ceremony or application test pass is implied here.

Use [requirements](REQUIREMENTS.md) for scope/acceptance,
[design](DESIGN.md) for contracts and [team tasks](TEAM_TASKS.md) for detailed work.

## Workflow decisions

The pasted proposal mixes a manager-led workflow with an earlier self-claim
prototype. The owner selected manager review/assignment for the first demo and
requester confirmation for P1 as well as other priorities.

| Area | Selected first milestone |
| --- | --- |
| Domain and roles | IT support; Employee, IT Manager and IT Team Member |
| Types and references | Service Request/Incident; UUID identity plus SR/INC sequence references |
| Ownership | Manager reviews; P1 assigned to acting manager, P2–P4 to eligible IT Member |
| Resolution | resolved means Awaiting employee confirmation; requester closes or reopens |
| Concurrency | Competing manager reviews/versioned changes, not open queue self-claim |
| Git integration | Keep reviewed feature -> develop -> main despite draft diagrams omitting develop |

HR needs a later separate access boundary. Comments, subcategories, reassignment,
reprioritization, dashboards, notifications, SLA automation and authentication
remain follow-up work. Manager review/initial assignment, reopen/close and events
are core; do not defer them while claiming the revised workflow is complete.

## Ownership and dependencies

| Member | Initial ownership | Created branch |
| --- | --- | --- |
| A | Users/role/status lifecycle and actor interface | feat/support-users |
| B: Shreyas | Shared foundation, typed categories and member eligibility | feat/support-categories |
| C | Ticket types/references, review/assignment, lifecycle/history | feat/support-tickets |

The branches were created from origin/develop at b56c317. The planning baseline
still needs review into develop, then normal synchronization into each branch.
Preserve B's uncommitted scaffold; use its own branch and working directory.

Implementation order:

1. B continues the small T-X00 Rust/config/DB/error/logging/OpenAPI/test/CI
   foundation PR; A/C review. A prepares users/actors while C writes pure
   priority, review and transition rules against the agreed contracts.
2. Decide the DB baseline before migrations. Existing external PostgreSQL data
   may remain; use a separate empty development database or explicit compatible
   path, without a silent reset.
3. Merge users -> typed categories/membership -> tickets/events with per-type
   reference sequences. Owners write service SQL; B coordinates names/order.
4. Integrate create -> manager review/assign -> start -> resolve -> requester
   confirm/reopen. A/B share fixtures, eligibility/race tests and documentation.
5. Complete A role/status guards and B membership-removal guards after C's
   schema exists; priority-specific assignee rules must remain true under races.

## Proposed three-day milestone

This is a revised capacity proposal, not a deadline guarantee. Keep five business
tables: users, categories, category_agents, tickets and ticket_events. Sequences
do not require another business table. Keep categories flat initially.

| Day | A | B | C | Checkpoint |
| --- | --- | --- | --- | --- |
| 1 | Role/user contract and create/get/list | Continue foundation; typed category contract/schema | TicketType/Priority/Status, review rules and schema | Shared contracts, reference allocation and migration order reviewed |
| 2 | User updates, fixtures and eligibility guards | Category CRUD and active IT Member membership | Ticket creation/listing and manager review/assignment | P1 manager and P2–P4 member paths demonstrated |
| 3 | Integration/access-scope tests and docs | Configuration/assignment race tests and setup notes | Resolution/root cause, requester confirm/reopen and history | Real-PG verification, OpenAPI, xh/jq demo and teammate review |

Review the changed scope before assigning capacity. Preserve version checks,
atomic history, required explanations and realistic test evidence when slicing.

P6 advises deferring authentication/authorization. A documented synthetic-actor
local demo can exercise role/business rules but does not prove employee identity
or confidential access. The company version still requires login/sessions,
revocation, backend scopes and permission tests. Never mark that work complete
because a role enum or supplied actor ID exists.

## Branching and review

```text
feature branch -> teammate-reviewed PR -> develop
             develop -> reviewed release PR -> main
```

In a clean clone, B checks out its existing remote branch:

```bash
git fetch origin
git switch --track origin/feat/support-categories
```

A/C substitute their branch. If local branches already exist, use git switch
with the local name without --track. Commit/preserve current work before changing
branches; do not use another member's dirty checkout.

From your feature branch after committing current work:

```bash
git fetch origin
git merge origin/develop
```

Resolve conflicts, run affected checks and push normally. Use focused PRs with
story IDs, API/schema effects, dependencies, exact-commit checks and remaining
limitations. Prefer merge commits for continuing service branches; no force
push, shared-history rewrite or direct main/develop push. Delete a feature branch
only when its work is integrated and no open PR depends on it.

Explicit owner-requested documentation updates may target main through a normal
PR without rule bypass. Application implementation continues through develop.
Recommended protections are one teammate approval, resolved conversations,
relevant CI once it exists, no direct/force pushes or deletion on main/develop,
and normal feature pushes with force pushes blocked. These are recommendations,
not a claim that repository settings enforce them.

## Agile records and definition of Done

Use the company's actual Agile framework/cadence; these are not specified.
Track Backlog -> Ready -> In progress -> Review/Test -> Done. Refine criteria and
dependencies, select an achievable slice, demonstrate it, record feedback and
choose one retrospective action with an owner.

Ready means owner, observable acceptance, contracts/dependencies and a small
slice are agreed. Done means behavior/OpenAPI match acceptance, meaningful
checks pass on the exact commit, a teammate reviews and deferred work is visible.
Record dates/goal, selected stories, owner/reviewer, blockers, PR/commit, check
results, demo feedback, unfinished scope and follow-up in the PR/existing board.
No extra progress document is required.

## Verification, release and maintenance

Core cases include typed-category uniqueness, category/type mismatch, inactive
users/categories, P1 manager/P2–P4 member eligibility, stable lists, reference
uniqueness/prefixes, required incident root cause, requester-only confirmation,
reopen reset/history, stale versions and rollback. Two manager reviews of one
version yield one success, one 409 and one review/assignment event. Test user/
membership changes racing review and former-handler history visibility.

Once implemented, run format, Clippy and unit/real-PostgreSQL tests with a
dedicated test DB. Demo creation -> manager review/assign -> start -> resolve ->
requester confirm/reopen, and inspect actual OpenAPI/logs with xh/jq. Planned
commands are not evidence of successful execution.

Before company release:

- [ ] Exact candidate passes checks, peer review and business acceptance.
- [ ] Runtime/config/migrations/OpenAPI/run instructions match the candidate.
- [ ] Real authentication and scoped authorization pass for all three roles.
- [ ] Operator/environment, first-manager setup/secrets and limitations are recorded.
- [ ] Backup/restore and migration recovery are rehearsed on a disposable database.

Record defects with reproduction, expected/actual result, affected story,
commit/environment and sanitized evidence. Agree priority, assign an owner,
add a regression check and review the fix through the same workflow.
