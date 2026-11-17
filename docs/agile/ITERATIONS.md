# Iteration planning, implementation and feedback

Follow the company's Agile cadence. Record whether its framework uses sprints,
the iteration length, meeting conventions and business reviewer during planning.
Until confirmed, the increments below are proposed goals without calendar promises.

## Proposed increments

| Increment | Goal | Candidate stories | Dependencies / acceptance |
| --- | --- | --- | --- |
| Foundation/inventory | Start the API and register/find/update equipment safely | US-B01..04 | B baseline exists; review and shared develop integration still need recorded evidence |
| Company electronics/location | Register all initial device types and distinguish portable/shared/installed deployment | US-B01/02/05 | Categories work now; location/usage contract, migration, eligibility and tests are the next increment |
| Identity and first handover | Authenticate the right actor and issue eligible portable equipment | US-A01..03, US-C01 | Shared schema/guard contracts and US-B05 eligibility; A/C can work in parallel after agreeing interfaces |
| Returns and clearance | Complete physical return, history and employee-owned reporting | US-C02..04 | Version/condition and authorization integration; rollback and concurrency evidence |
| Integrated acceptance/release | Demonstrate the complete outcome and operational readiness | US-X01, NFRs | All selected criteria, exact-commit tests/CI, review and release checklist |

Do not commit an entire increment to a fixed date before agreeing capacity,
story size and dependencies. Scope added mid-iteration is discussed with the
business reviewer: swap/replan work explicitly rather than silently expanding it.

## Working cycle

1. **Refinement:** write user value, criteria, data/access rules, dependencies and
   test approach. Split oversized stories; link existing work instead of duplicating it.
2. **Planning:** agree one demonstrable goal, select ordered stories/tasks, name
   owners/reviewers and record capacity/risks using the company's estimation practice.
3. **Daily coordination:** use the company stand-up to inspect evidence, current
   work and blockers. Help review/unblock before adding more unfinished work.
4. **Implementation:** design enough for the next story, code/test in a small PR,
   update OpenAPI/migrations/docs and use the agreed shared file ownership.
5. **Review/demo:** show working behavior against criteria and record accepted,
   rejected or deferred results, feedback and follow-up cards.
6. **Retrospective:** record what helped/hindered delivery and one actionable
   improvement with an owner and follow-up date agreed by the team.

## Definition of Ready

A selected item has a story/requirement ID, user-visible outcome, acceptance
criteria, owner, reviewer, dependency and API/data/security impact, and a test
plan. Important questions are resolved or recorded as a bounded discovery task.

## Definition of Done

- Acceptance criteria implemented, including relevant negative/race cases.
- Validation, permissions/ownership and transaction behavior reviewed.
- Formatting/Clippy/relevant tests pass for the actual commit; real DB tests do
  not silently skip when PostgreSQL is absent.
- OpenAPI, traceability, setup and changed design decisions are updated.
- Different teammate reviews; green PR checks and acceptance evidence recorded.
- PR merged into develop; remaining scope has separate backlog cards.

Done for an increment is not Released. Release needs candidate acceptance and
operating evidence in [RELEASE.md](RELEASE.md). An incomplete story is carried
forward explicitly, not counted as complete from partial work or a demo script.

## Git and shared integration

```text
feature/fix/docs branch -> reviewed PR -> develop -> release PR -> main
```

Feature branches: A feat/employees, B feat/assets-database, C
feat/assignments-clearance; access work may use a coordinated separate branch.
Use normal pushes and normal merges for dependencies; do not force-push shared
history. B's prerequisite schema/pool must land in develop before teammates rely
on it there. Each teammate merges the updated integration branch into their work.

A coordinates router/error/OpenAPI assembly; B coordinates schema/pool/CI;
each owner edits their feature/tests and contributes definitions for assembly.
Append new migrations after the baseline is shared/applied. Verify repository
branch settings separately; this written workflow does not enforce protection.

## Copyable iteration record

No meeting or sprint is marked conducted until this record is populated with
real participants/results. Keep records in this file or linked issue/PR documents.

```text
Iteration ID / company cadence / planned dates:
Participants and business acceptance reviewer:
Goal and selected ordered stories/tasks:
Capacity/estimates (company convention) and dependencies:
Daily updates: date, evidence completed, next work, blockers/owner:
Review: date, demonstrated commit/environment, criteria and observed outcomes:
Accepted / incomplete / deferred work; reviewer and evidence:
Feedback and resulting backlog/change IDs:
Retrospective: what helped, what hindered, improvement/owner/follow-up date:
```

## Initial recorded observation

This is an observation from the owner's API run, not an invented team review:
on 2026-11-16, /ready returned 200; an /asserts typo returned 404. xh POST input
with a space after asset_tag= failed in the client. Instructions were corrected
to quote complete key=value arguments. Confirm successful asset creation and the
remaining B manual criteria using [B acceptance](../B_ACCEPTANCE.md).
