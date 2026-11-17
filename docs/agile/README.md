# AssetDesk: Agile delivery handbook

Prepared 2026-11-17. This is the documentation baseline for the whole A/B/C
project, maintained alongside B's implementation branch for team review.
It does not claim that earlier meetings occurred, unfinished code passed, or
anyone approved a production release.

## Delivery approach

The company uses Agile. Work in small increments: refine a user story, agree
acceptance criteria and design, implement/test it, review a working result, and
adapt the backlog from feedback. Repeat these activities throughout development.
Documentation changes with the code in each PR.

Use the company's existing framework, iteration length and meeting cadence.
Those details and the business acceptance reviewer have not yet been supplied;
do not invent a Scrum role, sprint duration, velocity or completed ceremony.
The [Agile principles](https://agilemanifesto.org/principles.html) inform the
approach; the following working agreements are specific to this project.

## Lifecycle map

| Activity | Document / deliverable | Owner |
| --- | --- | --- |
| Discover purpose, users and boundaries | [Product brief](PRODUCT_BRIEF.md) | Team with business reviewer |
| Define requirements and acceptance | [Requirements](REQUIREMENTS.md), [stories/backlog](BACKLOG.md) | Each service owner |
| Refine and plan an increment | [Iteration workflow and records](ITERATIONS.md) | A/B/C together |
| Design API, data and transactions | [Architecture and decisions](DESIGN.md), OpenAPI, migrations | B coordinates; A/C review |
| Implement and review | [Ready/Done and Git policy](ITERATIONS.md), PR template | Feature owner + teammate reviewer |
| Verify correctness and acceptance | [Testing and traceability](TESTING.md), CI and demo record | Feature owner; C coordinates end-to-end |
| Review results and improve | [Iteration review/retrospective records](ITERATIONS.md) | Team and business reviewer |
| Release and hand over | [Release runbook](RELEASE.md), release notes/sign-off | B coordinates; all accept |
| Operate, fix and improve | [Maintenance, risks and changes](MAINTENANCE.md) | Assigned operator + team |

These are recurring activities within increments. The map is not a one-time
waterfall handoff or a claim that documentation alone completes the lifecycle.

## Evidence and current status

- B baseline: commit `2959a4c`, [PR #1 into develop](https://github.com/rootusershreyas/AssetDesk/pull/1).
  Historical validation: 7 unit and 11 PostgreSQL integration tests, formatting,
  Clippy and CI passed. Evidence applies to that commit.
- PR inspected 2026-11-16: open, mergeable, successful checks, no submitted reviews.
  This is a dated observation, not a live claim about later review/merge status.
- On 2026-11-16 the owner demonstrated `/ready` returning 200. The attempted
  `/asserts` request returned the expected route-not-found JSON; the correct
  route is `/assets`. A successful owner-run POST was not supplied as evidence.
- A's access-control work exists separately as a local draft and is not in the
  B baseline. Full employee management and C's issue/return remain pending
  integration. Authenticated end-to-end acceptance has not been recorded.
- This documentation change does not modify application code or rerun unchanged
  application tests. Document links/formatting are checked separately.
- The separate game/Collections learning review is outside AssetDesk's scope.

## How to use this package now

1. A/B/C review the scope and permission matrix; record the business reviewer and
   the company's actual planning/cadence conventions.
2. Refine the next stories and their dependencies using the backlog.
3. Record an iteration goal and selected work; name an owner and reviewer.
4. Build/review small PRs into develop, updating acceptance evidence and docs.
5. Demonstrate the increment, record feedback and a retrospective action.
6. Release only after the exact candidate satisfies the release checklist.

Supporting guides: [B acceptance](../B_ACCEPTANCE.md), [database contract](../B_DATABASE.md),
[B implementation walkthrough](../B_IMPLEMENTATION.md).

The included issue/PR templates are repository artifacts. Their availability as
GitHub defaults depends on merging them into the appropriate default branch;
copy the template content manually while that integration is pending. No remote
board, issue, ceremony or approval was created by writing these files.
