# IT support ticketing requirements

Status: planned, 2026-11-17. The shared repository contains documentation;
the owner has locally initialized B's Rust scaffold and reported cargo check
passing. No shared ticketing API or database workflow is implemented yet.
The final product name is undecided; AssetDesk remains the repository name.

## Product scope and agreed workflow

Employees raise IT **Service Requests** or **Incidents**. An **IT Manager**
reviews each ticket and selects P1/P2/P3/P4 priority. The manager personally
handles P1; P2–P4 go to an eligible **IT Team Member**. The assigned handler
records a resolution, and the original requester confirms closure or reopens.

The owner confirmed manager review/assignment for the first demo and requester
confirmation for every priority, including P1. These decisions resolve the
conflicting self-claim and manager-direct-closure flows in the pasted proposal.
Open queue claiming is not part of this selected workflow.

Initial domain: IT infrastructure support. Examples: a charger request is a
service_request; failed Wi-Fi is an incident. HR is outside the first workflow;
adding HR requires a separately authorized support boundary, not automatic
access for every IT Manager. Payroll/attendance processing is outside scope.

Fixed values:

- role: employee, it_manager, it_member.
- account_status: active, inactive.
- ticket_type: service_request, incident.
- priority: p1, p2, p3, p4; manager-decided, null before first review.
- status: open, in_progress, resolved, closed.

## First milestone and follow-up scope

The tentative three-day local API prototype uses five business tables: users,
categories, category_agents, tickets and ticket_events. PostgreSQL sequences
provide human ticket references without adding a business table.

Include the shared foundation; users/role metadata; type-aware flat categories;
member mappings; ticket creation/get/list; unique SR/INC references; manager
review and assignment; lifecycle, resolution, confirmation/reopening; events
and a minimal personal work-history projection. Lists support ticket_type,
category_id, status, priority and reviewed filters, chronological sorting and
bounded pagination (limit 20 by default, maximum 100; offset nonnegative).

Follow-up work: subcategories, comments/private notes, reassignment and changes
to an already-reviewed priority, advanced search/reporting, notifications,
attachments, SLA automation, dashboards/frontend and real authentication.
These remain requested target capabilities rather than completed prototype work.
Subcategories will use a one-level parent relationship in categories, inherit
root-category membership and stay within the same ticket_type.

The prototype may use explicitly documented synthetic actors locally. T-A02/03
login, sessions and backend authorization remain required before real company
use; role metadata alone does not establish identity or confidentiality.
The expanded business rules require new estimates; three days is a capacity
proposal, not an enterprise-release commitment. P1–P6 evidence remains mapped
in the root README; standalone algorithm exercises remain separate work.

## Roles and target access

| Operation | Employee | IT Team Member | IT Manager |
| --- | --- | --- | --- |
| Raise a ticket | For self | For self | For self |
| Read own requests and their resolution/history | Yes | Yes | Yes |
| Read another requester's full ticket | No | Only while currently assigned | All IT tickets |
| Review priority and assign | No | No | Yes |
| Handle P1 | No | No | Acting manager takes ownership |
| Handle P2–P4 | No | Assigned active member with category membership | Assign an eligible member |
| Start/resolve | No | Own assigned ticket | Own assigned P1 ticket |
| Confirm closure or reopen | Own resolved request | Own resolved request | Own resolved request |
| Manage users/categories/membership | No | No | Yes |
| Personal work history | Own request history | Own work metadata | Own work and all IT activity |

Being a manager is not a shortcut to confirming another person's resolution.
Every role may act as requester for its own ticket. A former handler's work
history exposes only that handler's historical metadata (reference, action,
time and own assignment/handling facts), not the ticket's ongoing contents or
other users' notes. Prior involvement alone grants no current full-ticket access.

Active users can discover the active category catalogue for creating requests.
Managers may include inactive categories in management views; deactivation does
not remove classification/history from existing tickets.

Target requester/actor IDs and roles come from validated server-side identity,
not client-selected privileged fields. Apply scope in SQL before filtering,
pagination or counts, including events and future comments. Missing/invisible
tickets return 404; forbidden actions on a visible resource return 403.

## Functional stories and acceptance

All business stories start in Backlog. B's local compiler check completes only
scaffold verification, not these HTTP/database acceptance criteria.

| ID | Owner | Acceptance |
| --- | --- | --- |
| T-X00 | B coordinates; A/C review | Establish Rust/runtime/configuration, ignore rules, DB baseline/pool/migrations, shared errors, actor interface, logging, OpenAPI, tests and CI. Preserve existing external DB data; agree an empty new DB or compatible migration path. |
| T-A01 | A | Create/get/list/update users with UUID, normalized unique email, name, role, account_status and timestamps. Validate inputs; duplicate email 409. Preserve referenced accounts. |
| T-A02 | A | Implement separately estimated login/logout, password setup/hashing, session expiry/revocation and inactive-user rejection. |
| T-A03 | A + B/C | Enforce the access matrix on all routes and nested/list/history paths; prove cross-user and previous-handler access restrictions. |
| T-B01 | B | Create/read/list/update categories with ticket_type, trimmed name 1..80 characters, description up to 1,000 and active flag. Names unique ignoring case within ticket_type; duplicates 409. Category type is immutable. |
| T-B02 | B | Deactivate/reactivate categories. Inactive categories block new requests; existing tickets can still be reviewed/handled. Preserve referenced categories. |
| T-B03 | B | Manage category membership idempotently. Only active it_members can join, and one member may support multiple categories. Block removal while that category has assigned non-closed work for the member. P1 manager eligibility is separate from membership. |
| T-B04 | B, later | Add one-level same-type subcategories, parent/child validation and inherited root membership; prevent parent changes/cycles and apply parent deactivation rules. Not in the first migration/API slice. |
| T-C01 | C | Create an open/unassigned/unreviewed ticket with UUID, immutable unique SR/INC ticket_number, ticket_type/category, subject 1..160, description 1..8,000, null priority, version 1 and timestamps. Active requester/category and matching category type required. |
| T-C02 | C | Scoped get/list with selected filters, stable created_at sorting (ID tie-breaker), limit/offset and minimal GET /api/v1/me/work-history from retained events. The manager can list reviewed=false to see pending review. |
| T-C03 | C | An active IT Manager reviews a pending open ticket with priority, eligible assignment and expected_version. P1 assigns the acting manager; P2–P4 require an active it_member in the category. Persist review metadata and one combined event atomically. Two competing reviews produce one success and one 409. |
| T-C04 | C | Validate lifecycle/handler/requester rules below. Resolve with explanation; incidents also require root cause. Retain atomic history and version checks; never auto-close a ticket. |
| T-C05 | C, later | Add employee-visible comments/private-note policy and related access/closure tests. |
| T-C06 | C, later | Manager reassignment/reprioritization is one atomic operation that revalidates role/category rules, version and history. Do not change priority alone and leave an ineligible assignee. |
| T-X01 | All | Integrate contracts, real-PG race/rollback tests, OpenAPI, sanitized logs and xh/jq demo. Record scope, identity limitations and exact-commit evidence. |

Any user status/role change that invalidates an assigned non-closed ticket is
blocked with 409 under the shared transaction lock. This applies to managers
holding P1 and members holding P2–P4, including a member promoted to manager.
Membership removal only governs member-assigned category work. Resolve/close
the work or, once the follow-up exists, use reviewed reassignment before removing
eligibility. Do not skip guards while waiting for the tickets schema.

## Review, assignment and lifecycle

A new ticket has status open, no assignee and no review stamps. Its priority is
null. POST /api/v1/tickets/{id}/review accepts priority, expected_version and an
assignee_id for P2–P4. For P1, the server uses the acting manager; a provided
assignee_id must equal that actor. Review leaves status open, sets reviewer/time
and assignee, increments version and appends a reviewed_and_assigned event.

| From | To | Actor and condition |
| --- | --- | --- |
| open | in_progress | Current assigned handler; manager review completed and eligibility rechecked |
| in_progress | resolved | Current assigned handler; nonblank resolution, plus root_cause for incident |
| resolved | closed | Original requester confirms; same rule for P1 and every other priority |
| resolved | open | Original requester reopens with a nonblank explanation |

The display label for resolved is **Awaiting employee confirmation**. Closed
means the requester accepted the resolution; it is final/read-only in this
increment. No open/in-progress ticket can jump directly to closed.

Reopening clears assignee, reviewed_by/reviewed_at and current resolution/root
cause, while retaining the last priority and all earlier values in events. It
returns to manager review. Therefore pending review must check open plus no
assignee/review stamps, not priority IS NULL. Re-review may select a new priority.

Resolution length is 1..8,000 characters; incident root_cause 1..4,000;
reopen explanation 1..4,000. Trim/validate nonblank content. Service requests
use a fulfillment explanation; root cause is optional. New ticket type,
requester, category, subject and description remain immutable initially.

## API, integrity and evidence

- Keep /api/v1 routes. Creates return 201 plus Location; reads/updates 200;
  successful membership PUT/DELETE 204. Malformed JSON 400; validation 422;
  unsupported content/body size 415/413; safe server/DB failures 500/503.
- Validate UUID references and unknown fields. After visibility checks, stale
  versions, ineligible assignments and conflicting reviews return 409 without
  partial updates or successful events. Authentication adds 401/403 behavior.
- Bind SQL values; enforce FK/unique/check constraints. Use PostgreSQL sequences
  for public references, not MAX()+1. Sequence gaps after rollback are acceptable.
- Version changes and their events commit together. Shared eligibility locking
  coordinates user/category/membership changes and ticket review/handling.
- Keep pure validation/lifecycle functions around async Axum/SQLx orchestration.
  Use real PostgreSQL for integration/concurrency tests and matching Utoipa
  OpenAPI for implemented routes. Proposed routes are not working endpoints.
- Trace request/ticket IDs, status and latency; exclude tokens, passwords,
  descriptions, root causes and resolution contents from operational logs.

P6 permits an alternative domain and focuses on practical HTTP API learning.
Its closing guidance defers authentication; the owner's requested company
access remains a separately verified requirement. No rubric score, production
acceptance, security or completed ticketing tests are claimed by this plan.
