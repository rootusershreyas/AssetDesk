# Project documentation

## Current direction: internal support ticketing

The owner changed the product direction on 2026-11-17 to employee support
tickets for Infra, HR and future categories. The final name is pending.
Ticketing is planned. The old executable source and build files have been
removed; this baseline contains plans and PostgreSQL setup only.

| Current guide | Purpose |
| --- | --- |
| [Project outline](../README.md) | Problem, workflow, implementation status and scope |
| [Detailed A/B/C assignments](ticketing/TEAM_TASKS.md) | Purpose, APIs, tables, files/functions, tests, first tasks and dependencies |
| [Branching workflow](ticketing/BRANCHING.md) | Created branches, normal synchronization, review and integration order |
| [Short-notice design and P1–P6 map](ticketing/SHORT_NOTICE_DESIGN.md) | Selected prototype scope, official product patterns and learning evidence |
| [Complete team handoff](ticketing/TEAM_HANDOFF.md) | End-to-end implementation sequence, module/function split, examples and recovery cases |
| [Ticketing requirements](ticketing/REQUIREMENTS.md) | Roles, acceptance criteria, lifecycle and P6/time constraints |
| [Ticketing design](ticketing/DESIGN.md) | Proposed modules, schema, HTTP contracts and transaction rules |
| [Ticketing delivery](ticketing/DELIVERY.md) | A/B/C tasks, B's first slice, Agile records, tests and release gates |
| [Decisions and cleanup](ticketing/CHANGES.md) | Current direction, milestone choices, removed files and retained data |

These are the current planning documents. They do not certify working tickets,
secure access, a production release or completed Agile ceremonies.

The ticketing delivery plan includes Agile planning, acceptance, review,
verification, release and maintenance records. Company cadence, business
reviewer, operator and final acceptance still need actual team records.

The superseded inventory code/docs remain in Git history. They are no longer
part of the active project files or evidence of a working ticketing API.
