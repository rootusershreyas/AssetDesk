# B: Agile acceptance and handoff

Scope: database setup and asset inventory on feat/assets-database. Account/role
management belongs to A; equipment issue/return and global clearance belong to C.
The shared business schema supports their integration without implementing those APIs.

CR-003 focuses inventory on company electronic devices/accessories. The existing
category contract supports all initial types; location/deployment and eligibility
are planned US-B05 work. See [device scope](DEVICE_INVENTORY_SCOPE.md). Current
manual availability checks apply to operational/unassigned custody, not proof
that a shared/installed device can be loaned.

## User stories and verification

The local labels B-01 through B-04 below map respectively to canonical
US-B01 through US-B04 in the [product backlog](agile/BACKLOG.md).

| Story | Acceptance criteria | Automated evidence in tests/assets.rs |
| --- | --- | --- |
| B-01: As an IT administrator, I want to register tagged equipment so I can identify company assets. | Trim/uppercase tag; validate required fields; create returns 201 and Location; duplicate normalized tag returns 409; get missing ID returns 404. | creates_normalizes_retrieves_and_rejects_duplicate_tags; validates_create_and_patch_without_losing_omitted_values; returns_consistent_errors_for_missing_and_malformed_requests |
| B-02: As an IT administrator, I want to find available equipment so I can choose an appropriate device. | Literal case-insensitive name/tag search; category/status/availability filters; deterministic sort and ID tie-break; default limit 20, maximum 100; nonnegative offset. | derives_availability_from_current_custody_and_filters_assets; searches_literal_wildcards_and_backslashes; sorts_and_paginates_with_stable_id_tie_breakers |
| B-03: As an IT administrator, I want safe asset updates so custody records remain consistent. | Tag immutable; omitted PATCH fields retained; null/unknown/empty updates rejected; assigned asset cannot enter maintenance/retired; rejected combined updates roll back all changes. | validates_create_and_patch_without_losing_omitted_values; rejects_assigned_asset_status_changes_atomically; status_change_waits_for_issue_lock_then_rechecks_custody |
| B-04: As a teammate, I want a reproducible database and API contract so I can integrate my service. | Compose/health check, migration, pool and readiness work; one outstanding assignment per asset; OpenAPI documents implemented routes and errors. | migration_enforces_custody_history_and_return_constraints; readiness_tracks_database_while_health_stays_live; openapi_documents_assets_filters_and_conflict_responses |

These stories describe domain actors. The standalone B baseline has no login
enforcement; A must integrate admin authorization when the access service lands.
Tests verify B's behavior using database fixtures until C's write APIs exist.

## Recorded baseline evidence

- Implementation commit: 2959a4c.
- Review PR: [#1 into develop](https://github.com/rootusershreyas/AssetDesk/pull/1).
- Current PR inspection, 2026-11-16: open, mergeable, no submitted reviews;
  both recorded Rust checks jobs successful.
- Historical checks for that baseline: 7 unit and 11 real-PostgreSQL integration
  tests passed, along with formatting, Clippy and live HTTP verification.
- [PR CI run](https://github.com/rootusershreyas/AssetDesk/actions/runs/43174568637).

An earlier passing run is evidence for its commit, not for later changed code.
Record new results when implementation changes. The manual checklist below is
unchecked because this document does not claim the user's demonstration passed.

## Manual API acceptance

Run from the B checkout. Keep cargo run in one terminal and use another for xh.
The .env file is already configured in this checkout; preserve local settings.

```bash
docker compose up -d --wait db
cargo run --locked
```

The commands below assume the default address. Choose a different asset tag if
the example tag already exists. In xh, key=value is one body-field argument;
key==value is a query parameter. There must be no space after the equals sign.

- [ ] Check database readiness: expect 200 and status ready.

```bash
xh GET http://127.0.0.1:3000/ready
```

- [ ] Register a sample asset: expect 201, a UUID and Location.

```bash
xh POST http://127.0.0.1:3000/api/v1/assets "asset_tag=B-DEMO-002" "name=B demo laptop" "category=laptop"
```

- [ ] Repeat the same POST: expect 409 duplicate_asset_tag.
- [ ] Copy the UUID from the successful response, then retrieve the asset:

```bash
asset_id="REPLACE_WITH_RETURNED_UUID"
xh GET "http://127.0.0.1:3000/api/v1/assets/$asset_id"
```

- [ ] Search and paginate: expect matching items plus limit/offset.

```bash
xh GET http://127.0.0.1:3000/api/v1/assets q==B-DEMO category==laptop sort==asset_tag order==asc limit==5 offset==0
```

- [ ] Change status on an unassigned asset: expect maintenance and available false.

```bash
xh PATCH "http://127.0.0.1:3000/api/v1/assets/$asset_id" "operational_status=maintenance"
```

- [ ] Restore in_service: expect available true while no assignment exists.

```bash
xh PATCH "http://127.0.0.1:3000/api/v1/assets/$asset_id" "operational_status=in_service"
```

- [ ] Invalid pagination: expect 422.

```bash
xh GET http://127.0.0.1:3000/api/v1/assets limit==101
```

- [ ] Inspect OpenAPI:

```bash
xh --body GET http://127.0.0.1:3000/api-docs/openapi.json | jq '.paths | keys'
```

Scripted/noninteractive xh invocations should add --ignore-stdin unless JSON is
intentionally being piped into the request. Tests for assigned-asset rejection
and competing transactions are automated; the full issue/return demo waits on C.

## Checks and handoff

When code changes, run with PostgreSQL available:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

- [ ] Another teammate reviews the B PR and acceptance evidence.
- [ ] Merge B into develop through the reviewed PR.
- [ ] A/C integrate the shared schema and pool from develop using normal merges.
- [ ] A adds account guards and documents expected 401/403 responses.
- [ ] C follows employee -> asset -> assignment locks; B status updates already
  acquire the asset lock before checking outstanding custody.
- [ ] C uses the same asset status strings and one_open_assignment_per_asset index.
- [ ] New shared schema changes append migrations; do not edit the applied baseline.
- [ ] Complete a team issue/return/clearance test before the release PR to main.

Normal pushes and reviewed PRs remain the integration workflow. A written
checklist does not itself configure branch protection or authorize a release.
