# Maintenance, risks and change control

## Feedback and defect flow

Capture user feedback or a defect as a backlog item with reproduction, expected
versus actual behavior, affected story, environment/commit and sanitized evidence.
Agree business severity/priority with the reviewer, assign an owner, implement a
regression test/fix through the normal PR workflow and verify the result.
Decide whether the fix needs an expedited release or the next planned increment.

Inspect readiness failures, error rates/latency, migration failures and security
events without logging credentials. Use the actual environment's operational
process for outages. Dependency and access reviews, backup retention and restore
rehearsals need a named operator/cadence before organizational use.

## Risk register

| Risk | Impact | Owner / mitigation | Recorded status |
| --- | --- | --- | --- |
| Added access scope exceeds original time budget | Incomplete features or rushed security | All: re-estimate, prioritize stories, defer optional work, review auth lifecycle | Open |
| A/C start without B shared foundation | Duplicate schemas/interfaces and merge conflicts | B/all: review B prerequisite into develop; merge shared contract before integration | Open dependency |
| Issue/status/return races | Conflicting custody or partial state | B/C: consistent locks, transactions, unique index and real overlap/rollback tests | B side tested; full C integration pending |
| Incomplete authorization/ownership integration | Employee data exposed or unauthorized writes | A + each owner: shared guards, server-derived identity, per-endpoint negative tests | Open integrated-release gate |
| Editing applied migrations or deleting volumes | Startup failure or data loss | B: append migrations, preserve data, rehearse backup/restore | Procedure documented; restore unverified |
| Old green evidence used for changed code | False readiness assessment | All: exact-commit test/CI/review records and honest unfinished-story status | Process required |
| Review/acceptance responsibilities unclear | Work merged/released without acceptance | All: name business reviewer/operator; teammate review and explicit release record | Names/confirmation pending |

## Change record: CR-001 access control

- Origin: owner requested admin/user management after the initial auth deferral.
- Accepted scope direction: add authentication, account roles, session lifecycle
  and employee-owned reads. Detailed design/implementation acceptance is pending.
- Affected: US-A02/03, employee deactivation integration, B/C permission checks,
  self-service data ownership, schema, tests, OpenAPI and operator setup.
- Impact: extra implementation/review and revised planning; no updated delivery
  date or capacity commitment was recorded. B's standalone baseline is unchanged
  by documenting the requirement.
- Next action: A reviews/stabilizes the access draft; owners agree integration
  criteria and refresh remaining estimates at planning.

## Change record: CR-002 lifecycle documentation

- Origin: owner requested company Agile methodology and documentation from start
  through release/maintenance.
- Delivered artifact: this documentation package, local backlog/evidence records
  and review/issue templates.
- Impact: documentation/review workflow; no application runtime/schema changes.
- Acceptance: team/business review pending. Writing plans does not prove past
  ceremonies occurred or future release checks passed.

## New change/incident template

```text
ID, date, reporter, owner:
Reason/user impact and affected story IDs:
Expected/actual behavior or requested outcome:
Environment/commit, reproduction and sanitized evidence:
Priority/severity and business reviewer:
API/schema/access/design/test/documentation impact:
Options, dependencies, capacity/date implications:
Decision: accept/defer/reject; who and when:
PR, regression test, CI and acceptance evidence:
Release/recovery outcome and follow-up actions:
```

Revisit open risks and improvement actions in refinement/retrospectives. Update
requirements, design and release notes with accepted changes rather than silently
expanding scope or marking unverified work complete.
