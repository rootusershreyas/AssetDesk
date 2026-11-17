# Delivery, Agile and Git workflow

Planning baseline: 2026-11-17. Final product name pending. The repository
contains Markdown documentation only; application code, runtime configuration,
Compose, migrations, tests and CI still need implementation. Plans do not
establish completed ceremonies, acceptance or a working application.

Use [requirements](REQUIREMENTS.md) for scope/acceptance,
[design](DESIGN.md) for API/data/transaction contracts and
[team tasks](TEAM_TASKS.md) for each member's files, functions and first PR.

## Ownership and dependencies

| Member | Initial ownership | Created branch |
| --- | --- | --- |
| A | Users and account lifecycle; identity interface | feat/support-users |
| B: Shreyas | Shared foundation coordination, categories and agent membership | feat/support-categories |
| C | Tickets, claiming, lifecycle and history | feat/support-tickets |

The three branches were created from origin/develop at b56c317. Their initial
contents are the earlier documentation baseline. Main already holds the updated
project documentation; bring the planning baseline into develop through review,
then merge updated origin/develop into each service branch.

Implementation order:

1. B coordinates T-X00: a small Rust/config/DB/error/logging/OpenAPI/test/CI
   foundation PR, reviewed by A/C. A can prepare user DTOs/validation while C
   prepares pure lifecycle rules and tests against the agreed contracts.
2. Agree the database baseline before applying migrations. Existing external
   PostgreSQL data/volumes may remain; use a separate empty development database
   or a reviewed compatibility path rather than silently resetting data.
3. Merge schema in dependency order: users, categories/membership, tickets/events.
   Each owner authors their service SQL; B coordinates migration names/order.
4. Integrate ticket creation, eligible-agent claim, lifecycle and atomic events.
   A/B help C with fixtures, race tests and the end-to-end demonstration.
5. Complete A's eligibility-removing role/status guards and B's membership-removal
   guards only after C's ticket schema exists and integrated checks pass.

## Proposed three-day milestone

This is a capacity proposal including foundation work, not a delivery guarantee.
The prototype uses users, categories, category_agents, tickets and ticket_events.

| Day | A | B | C | Checkpoint |
| --- | --- | --- | --- | --- |
| 1 | User contract, schema and create/get/list | Foundation coordination; category schema and create/get/list | Ticket types, pure rules and schema contract | Reviewed shared contracts and ordered migration plan |
| 2 | User updates/guards and fixtures | Category activation and agent membership | Ticket create/get/list and eligible-agent claim | Synthetic request reaches the correct queue; one successful claim |
| 3 | Integration tests and documentation | Test races, integration and setup notes | Lifecycle, version checks and event history | Real-PG tests, matching OpenAPI, CLI demo and teammate review |

Comments, admin reassignment, advanced queries and authentication remain
follow-up work. Record partial T-C02/T-C03 slices without marking their full
parent stories Done. Preserve validation, transaction integrity and meaningful
concurrency tests when reducing scope.

P6 recommends deferring authentication/authorization. A local synthetic-user
prototype can demonstrate business rules; role metadata or a demo actor selector
does not prove identity or HR privacy. T-A02/03 remain company-use requirements,
including sessions, revocation and cross-user/category access tests.

## Branching and review

```text
feature branch -> teammate-reviewed PR -> develop
             develop -> reviewed release PR -> main
```

Use a clean checkout. For B, whose remote branch already exists:

```bash
git fetch origin
git switch --track origin/feat/support-categories
```

A/C substitute their branch from the ownership table. If the local branch
already exists, use git switch with its local name, without --track. Preserve
uncommitted work before switching; do not use another member's dirty checkout.

From your feature branch, after committing current work:

```bash
git fetch origin
git merge origin/develop
```

Resolve conflicts, run affected checks, commit any resolution and push normally.
Use small PRs with story IDs, API/schema effects, dependencies, exact-commit
verification and remaining limitations. Another teammate reviews before merge.
Prefer merge commits for this milestone's continuing service branches; do not
rewrite shared history, force-push or directly push to main/develop. Delete a
service branch when its milestone work is integrated.

Owner-requested documentation-only updates to main use a normal PR containing
only the requested documentation, without rule-bypass flags. Implementation
continues through develop. No release or protection enforcement is implied.

Recommended rules: require one teammate approval and resolved conversations on
main/develop, block direct/force pushes and deletion there, and allow normal
feature pushes while blocking force pushes. Add required CI checks only after
the workflow exists and has a known successful run. These are recommendations;
repository settings must be verified/configured separately.

## Agile working agreement

Use the company's actual Agile framework and cadence; neither is specified yet.
Track stories through Backlog -> Ready -> In progress -> Review/Test -> Done.
Refine acceptance/dependencies, select an achievable increment, demonstrate it,
record feedback and choose one retrospective improvement with an owner.

Ready means a story has an owner, observable criteria, agreed dependencies and
a small implementation slice. Done means its acceptance matches the working
API/OpenAPI, relevant checks pass for that commit, a teammate reviews it and
documentation/evidence is current. Deferred company-access work stays visible.

For each increment record: dates/goal, selected story IDs, owner/reviewer,
blockers, PR/commit, verification results, demonstration feedback, unfinished
scope and one improvement/follow-up. Keep these records in the relevant PR or
team's existing board; a separate progress file is not required.

## Verification, release and maintenance

Core acceptance must cover invalid inputs and duplicate names/emails, inactive
category/user behavior, membership eligibility, stable filtering/pagination,
valid/invalid transitions, stale versions, rollback on event failure and two
competing claims yielding one success, one 409 and one successful claim event.
Test configuration-change/assignment races once the shared schema is integrated.
Company use additionally requires real-session and forbidden-access cases.

Once the Rust foundation exists, run formatting, Clippy and tests against a
dedicated real PostgreSQL test database, then demonstrate create -> claim ->
start -> resolve -> reopen/close with xh/jq and inspect actual OpenAPI/logs.
These application checks cannot run from today's documentation-only baseline.

Before a release:

- [ ] Exact candidate passes required checks, teammate review and business acceptance.
- [ ] Runtime/configuration/migrations/OpenAPI/run instructions match the candidate.
- [ ] Authentication/authorization pass before real employee or HR use.
- [ ] Operator/environment, secure bootstrap/secrets and known limitations are recorded.
- [ ] Backup/restore and migration recovery are rehearsed on a separate disposable database.

Capture defects with reproduction, expected/actual behavior, affected story,
commit/environment and sanitized evidence. Prioritize with the reviewer, assign
an owner, add a regression check and review the fix through the same PR flow.
