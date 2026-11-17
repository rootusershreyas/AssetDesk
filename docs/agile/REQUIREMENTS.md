# Requirements specification

Scope baseline: company IT/electronic-device inventory (CR-003) plus the owner's
admin/user-access addition. Name pending; AssetDesk is the technical working name.
See [device scope and catalogue](../DEVICE_INVENTORY_SCOPE.md).

The functional requirement IDs are the user-story IDs in [BACKLOG.md](BACKLOG.md).
Their acceptance criteria are canonical; link changes to the affected IDs rather
than maintaining conflicting copies of the same behavior.

## Functional areas

| Area | Requirement/story IDs | Responsible owner |
| --- | --- | --- |
| Employee lifecycle | US-A01 | A |
| Authentication/session lifecycle | US-A02 | A |
| Account roles and administration | US-A03 | A |
| Asset creation/details | US-B01 | B |
| Inventory queries | US-B02 | B |
| Transactional asset updates | US-B03 | B |
| Database/runtime foundation | US-B04 | B with shared integration |
| Shared/installed location and issue eligibility | US-B05 (planned) | B, coordinated with C |
| Equipment issue | US-C01 | C |
| Versioned physical return | US-C02 | C |
| Custody collections/history | US-C03 | C |
| Clearance/overdue reporting | US-C04 | C with A's ownership guards |
| Complete integrated acceptance | US-X01 | All |

## Actors and permission requirements

| Action | Anonymous | User | Admin |
| --- | --- | --- | --- |
| Login, health/readiness, specification | Yes | Yes | Yes |
| Own profile/password/session operations | No | Yes | Yes |
| Own history and equipment clearance | No | Yes | Yes |
| Account/employee/global inventory management | No | No | Yes |
| Issue/return and other-employee records | No | No | Yes |

This matrix describes the required integrated product. The standalone B branch
does not implement login guards; A's access integration must enforce them.
Derive identity/ownership from the session. Client-supplied roles or owner IDs
must not grant access. Return 401 for missing/invalid sessions and 403 for a valid
identity without permission.

## Data and business constraints

- UUID identifiers; timestamps with time zones and consistent UTC reporting.
- Unique normalized employee emails; active/inactive employee status.
- Unique trimmed/uppercase immutable asset tags; asset states in_service,
  maintenance and retired. Name/category are validated, nonempty bounded text.
- Initial suggested categories: tv, tv_remote, laptop, hdmi_connector, wifi_device,
  laptop_charger, ups. More categories require no code/schema change; each physical
  unit/accessory is individually tagged. No aggregate stock/quantity model.
- Availability is derived from operational state and outstanding assignments.
- Current available is not installed-device issue eligibility. US-B05 must add
  location/deployment rules; do not infer portability from category alone.
- A device has at most one outstanding assignment; returned records remain.
- Assignment employee/asset IDs are immutable. Transfer means return then new issue.
- Return timestamp and condition are both absent or both present; versions positive;
  expected return after issue; return not before issue.
- Inactive employees cannot receive new issues but existing custody can be returned
  by an administrator. Deactivation does not delete history.
- Deadlines mark unfinished custody overdue; they never automatically return,
  reissue, disable or stop equipment.
- Clearance means zero outstanding assignments, including a never-borrowed case.

## Nonfunctional acceptance

| ID | Requirement / evidence |
| --- | --- |
| NFR-01 | Typed JSON/HTTP contract with consistent errors and OpenAPI for actual implemented routes; validate malformed, missing, null and unknown inputs |
| NFR-02 | Database constraints, short transactions and consistent locks prevent conflicting committed custody; test overlapping requests and rollback |
| NFR-03 | Passwords salted/hashed with Argon2id; only hashes of random session tokens stored; expiry/revocation/role/ownership enforced server-side; no secret logging |
| NFR-04 | Async I/O and pooled DB connections; blocking password work bounded/offloaded; readiness DB query timeout two seconds; liveness independent of DB |
| NFR-05 | Structured operation/request logs and no-store responses for protected/session data; inspect failures without exposing raw SQL/credentials |
| NFR-06 | Repeatable PostgreSQL migrations/setup; isolated real-DB integration tests plus meaningful unit tests, formatting, Clippy and CI |
| NFR-07 | Reviewable normal-push PR workflow; feature acceptance, release notes, backup/restore and operator handoff recorded before release |

No throughput, latency percentile, employee-count capacity or availability SLA
has been agreed or benchmarked. Define a workload and measurable target before
making such claims. Production infrastructure/security configuration is not
implied by running the local prototype.

## Requirement changes

Owner requests become change records in [MAINTENANCE.md](MAINTENANCE.md). Record
reason, impacted stories/design/tests, capacity implications and decision. Update
the backlog before marking changed work accepted. Authentication's original P6
deferral is superseded by the owner's explicit scope addition for this project.
