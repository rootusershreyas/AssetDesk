# Product backlog and task board

Prepared 2026-11-17 from known code/evidence. Priority order is a proposal for
team/business refinement. No GitHub Project or issues are claimed to exist from
this document alone. Each story must be accepted by its assigned owner/reviewer.

## Stories and acceptance criteria

| ID / owner | User story | Acceptance criteria | Recorded state |
| --- | --- | --- | --- |
| US-A01 / A | As an administrator, I want employee records so custody can be associated with active people. | Create/get/list/update/deactivate; unique normalized email; retain history; deactivation/session/last-admin policy integrated. | Full API pending |
| US-A02 / A | As an employee, I want to log in and end sessions so access is personal and revocable. | Valid login creates expiring bearer session; generic invalid-credential errors; logout revokes; disabled/inactive/expired identities rejected; password change revokes old sessions. | Separate local draft; not in B baseline |
| US-A03 / A | As an administrator, I want to manage accounts/roles so permissions match responsibilities. | Admin-only create/link/list/get/update; no self-escalation; changes invalidate sessions; final active admin protected even under concurrent administration. | Separate local draft; acceptance pending |
| US-B01 / B | As IT, I want to register and retrieve tagged equipment so each device is identifiable. | Normalized unique tag; valid fields; 201/Location; duplicate 409; missing asset 404; immutable tag. | Implemented/tested baseline; review evidence below |
| US-B02 / B | As IT, I want searchable inventory so I can find suitable equipment. | Literal case-insensitive search; category/status/availability filters; allowed sort and stable ID ties; limit 1..100/nonnegative offset. | Implemented/tested baseline |
| US-B03 / B | As IT, I want safe status updates so equipment in use is not marked unavailable. | Omitted fields retained; null/unknown/empty updates rejected; assigned maintenance/retired change rejected; accompanying metadata rolls back; concurrent issue rechecked after lock wait. | Implemented/tested baseline |
| US-B04 / B | As a teammate, I want a reproducible database/runtime so I can integrate my service. | Compose health, shared migration/pool/readiness, constraints and OpenAPI; /health remains live when DB unavailable. | Implemented/tested baseline |
| US-B05 / B with C | As IT, I want location and deployment mode so installed/shared devices are tracked without being treated as employee loans. | Location/usage contract agreed; new migration and API validation/filtering; installed/shared versus assignable eligibility explicit; tests cover all seven categories and a future category; existing records handled by reviewed backfill. | Planned CR-003 increment; no new fields implemented |
| US-C01 / C | As IT, I want to issue portable equipment so custody is recorded without conflicting handovers. | Active employee + eligible assignable in-service asset; obey US-B05 deployment rules before covering installed/shared types; valid optional deadline; overlapping requests yield one 201, one 409 and one open row. | Full API pending; coordinate US-B05 |
| US-C02 / C | As IT, I want to record physical returns so inventory and history stay consistent. | Current version required; repeated/stale returns 409; server time/condition recorded; damaged return + maintenance atomic; return from inactive borrower allowed. | Full API pending |
| US-C03 / C | As IT, I want custody history so I can inspect device/employee handovers. | Filter employee/asset/state/overdue; sort/paginate; preserve returned records; user-owned read queries cannot select another owner. | Global API pending; self-read draft separate |
| US-C04 / C | As an employee, I want an equipment-clearance report so I know what to return. | Own records only; zero outstanding means clear; overdue excludes returned/no-deadline items; empty/missing/boundary cases tested; no automatic deadline return. | Full acceptance pending |
| US-X01 / all | As the business reviewer, I want a complete reproducible demonstration so I can accept the increment. | Clean setup/login; register initial electronic types; verify shared/installed location and portable eligibility after US-B05; portable issue/conflict/return/history/clearance; exact-commit CI/review and limitations. | Integration pending |

## Board policy

```text
Backlog -> Ready -> In progress -> Review/test -> Done
```

Ready requires purpose, criteria, owner, dependencies and a test plan. Start the
clock at In progress; finish at Done after review and merge into develop.
Blocked is a flag on the current state with cause/next action, not a completion
state. Proposed small-team practice: one lead item per member; help review or
unblock before pulling more. Follow the company's actual work limits/metrics.

## Initial work cards

| Card | Story | Owner | State / next action |
| --- | --- | --- | --- |
| TASK-B-REVIEW | US-B01..04 | B; A reviews | Review/test: PR #1 was open with green checks and no reviews when inspected 2026-11-16; confirm/complete review before merge |
| TASK-DOC-REVIEW | All | B coordinates, A/C review | Review/test: this documentation baseline needs business/team confirmation and cadence details |
| TASK-B-DEVICE-SCOPE | US-B01/02 | B; A/C review | Review/test: document seven initial electronic-device categories, extensibility and CR-003; name pending |
| TASK-B-LOCATIONS | US-B05 | B with C | Backlog: agree location/deployment model and migration; distinguish installed/shared infrastructure from issuable portable items |
| TASK-A-ACCESS | US-A02/03 | A | Draft/in progress in separate checkout; stabilize build, review security, retest actual candidate |
| TASK-A-EMPLOYEES | US-A01 | A | Backlog: employee API and access/session integration |
| TASK-C-ISSUE | US-C01 | C | Backlog: integrate B prerequisite and implement issue transaction/tests |
| TASK-C-RETURN | US-C02 | C | Backlog: versioned return and damaged-return transaction/tests |
| TASK-C-REPORTS | US-C03/04 | C with A | Backlog: integrate history/self-service/global reports with ownership checks |
| TASK-INTEGRATION | US-X01 | All; C coordinates | Backlog: end-to-end acceptance after service integration |
| TASK-RELEASE | US-X01 / NFR-07 | All; B coordinates | Backlog: release and restore evidence, notes, acceptance and handoff |

No card is marked Done merely because these documents were written. B's checked
implementation and its PR acceptance are different facts. Start/finish dates,
cycle times, estimates and velocity have not been retrospectively invented.

## Task/bug record

```text
ID, story/change ID, owner, reviewer:
Outcome and acceptance criteria:
Dependencies, API/schema/access impact:
Stage, started/finished dates, blocker and next action:
Branch, exact commit, PR:
Commands/results and CI evidence:
Reviewer/acceptance and remaining limitations:
```
