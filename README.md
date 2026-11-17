# AssetDesk (working name)

A Rust REST API for **company IT infrastructure and electronic-device inventory**.
The final project name will be chosen by the owner later.

Track TVs, TV remotes, laptops, HDMI connectors, Wi-Fi devices, laptop chargers,
UPS units and more company electronic device types as needed.

## Goal and scope

Answer: **What electronic equipment does the company own, what type is it, and
what is its operational status?** The integrated product will also show where
equipment is deployed and who is responsible for portable items.

Inventory is the main purpose. Employee issue/return supports portable equipment;
installed/shared devices need location/deployment tracking. This is physical
inventory management, not automatic network monitoring or remote device control.
See the [device inventory scope](docs/DEVICE_INVENTORY_SCOPE.md).

## Initial device catalogue

| Device | Suggested category | Example tag |
| --- | --- | --- |
| TV | tv | TV-001 |
| TV remote | tv_remote | REM-001 |
| Laptop | laptop | LAP-001 |
| HDMI connector/adapter | hdmi_connector | HDMI-001 |
| Wi-Fi device | wifi_device | WIFI-001 |
| Laptop charger | laptop_charger | CHG-001 |
| UPS | ups | UPS-001 |

Category is extensible text. Add usb_hub or projector without changing code or
schema. Use consistent spelling for exact filters. Each physical unit gets its
own tag; there is no aggregate quantity/stock model in the current API.

[examples/devices.json](examples/devices.json) contains seven synthetic POST
bodies, not actual company inventory. Nothing is imported automatically.

## Implementation status

| Area | Status |
| --- | --- |
| Database/pool and device create/get/list/PATCH | Implemented in B baseline |
| Unique tags, categories, operational states, search/filter/sort/page | Implemented |
| Transactional status guard for employee-assigned equipment | Implemented |
| Admin/user accounts and authorization | A's separate integration work |
| Portable issue/return/history and clearance | C's integration work |
| Structured location and deployment mode | Planned US-B05; not current fields |
| Parent-device/accessory links and maintenance/movement history | Optional later backlog |
| Final product/repository/package rename | Owner decision pending |

Current available means in_service with no outstanding employee assignment. It
does not prove that an installed TV/UPS may be borrowed. The next location/
deployment increment must distinguish those concepts before issue operations
cover all device types. Category alone does not determine deployment.

## Run B's inventory API

From a fresh checkout, create .env only if one does not already exist:

```bash
cp .env.example .env
docker compose up -d --wait db
cargo run --locked
```

Preserve existing local configuration. Technical names remain assetdesk while
branding is undecided: PostgreSQL at 127.0.0.1:5434, API at 127.0.0.1:3000.
Startup connects and applies committed migrations before serving. SQLx CLI is
optional. Exported environment variables override .env.

Use RUST_LOG=info,tower_http=debug for request logs. In scripted xh commands add
--ignore-stdin unless intentionally piping JSON into the request.

```bash
xh GET http://127.0.0.1:3000/ready
xh POST http://127.0.0.1:3000/api/v1/assets "asset_tag=TV-001" "name=Meeting room TV" "category=tv"
xh GET http://127.0.0.1:3000/api/v1/assets category==tv sort==asset_tag order==asc limit==20
xh GET http://127.0.0.1:3000/api-docs/openapi.json
```

Keep key=value as one argument without a space after equals. Query parameters
use key==value. Reusing an existing tag returns 409.

## Current endpoints and data

| Method | Endpoint | Purpose |
| --- | --- | --- |
| POST | /api/v1/assets | Register device/accessory |
| GET | /api/v1/assets | Search/filter/sort/paginate inventory |
| GET | /api/v1/assets/{id} | Read device details |
| PATCH | /api/v1/assets/{id} | Update metadata or operational status |
| GET | /health | Process liveness |
| GET | /ready | Database readiness |
| GET | /api-docs/openapi.json | Implemented API contract |

Create accepts asset_tag, name, category and optional operational_status. Responses
add UUID/timestamps and available. Tags are immutable. PATCH accepts name/category/
status: omitted values remain; null, unknown fields and empty updates are rejected.
Location/usage/parent-device IDs are not accepted yet.

Statuses: in_service, maintenance, retired. They record inventory state, not live
hardware health. Collections support literal name/tag search (q), category,
operational_status and availability filters; sorts are asset_tag, name, created_at.
Limit defaults to 20, maximum 100; offset is nonnegative; ID breaks sorting ties.

The shared business migration creates employees, assets and assignments with
UUIDs, time-zone-aware timestamps, constraints and a partial unique index allowing
only one outstanding assignment per device. Keep applied migrations immutable;
location/deployment changes need an additional migration and tests.

## A/B/C ownership

| Member | Service | Branch |
| --- | --- | --- |
| A | Employees, accounts, admin/user access and guards | feat/employees / coordinated access branch |
| B: Shreyas | Device/accessory inventory, categories, DB and planned location/deployment | feat/assets-database |
| C | Portable allocations/returns/history/reports; shared/installed deployment history next | feat/assignments-clearance |

A supplies authentication; B/C apply guards and ownership in the integrated
product. The standalone B API does not enforce login yet.

Custody operations lock employee -> asset -> assignment. C locks the asset before
checking issue eligibility and inserting custody. B locks before status checks
and rolls back invalid combined updates. Preserve history; use versioned returns;
damaged returns set maintenance atomically. Deadlines never automatically return
or disable physical equipment.

## Agile delivery and documentation

Follow company Agile practice: refine stories/criteria, plan an increment,
design/build/test, review working behavior and adapt from feedback. Company
framework/cadence and business acceptance reviewer still need recording.

- [Documentation index](docs/README.md) and [Agile handbook](docs/agile/README.md)
- [Product brief](docs/agile/PRODUCT_BRIEF.md)
- [Requirements](docs/agile/REQUIREMENTS.md) and [backlog](docs/agile/BACKLOG.md)
- [Iterations and Ready/Done](docs/agile/ITERATIONS.md)
- [Architecture/decisions](docs/agile/DESIGN.md)
- [Test traceability](docs/agile/TESTING.md)
- [Release/recovery](docs/agile/RELEASE.md) and [maintenance/changes](docs/agile/MAINTENANCE.md)
- [B setup](docs/B_DATABASE.md), [implementation](docs/B_IMPLEMENTATION.md) and [acceptance](docs/B_ACCEPTANCE.md)

CR-003 records this scope change. Location/deployment and optional relationships
are planned, not implemented. Documentation does not certify final acceptance.

## Checks and integration

With dedicated PostgreSQL available:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Integration tests use isolated real PostgreSQL and do not silently skip missing DB
access. CI runs formatting, Clippy and tests. Historical evidence belongs to its
exact commit; changed-code results belong in its PR. Scope/example changes alone
do not certify future location or C workflows.

```text
feature/fix/docs -> reviewed PR -> develop -> release PR -> main
```

Use normal pushes and teammate review. Merge B's shared foundation before A/C
depend on it in develop. PR/issue templates link stories, design, tests and
acceptance; branch protections require separate repository configuration.
