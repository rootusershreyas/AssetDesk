# B: assets implementation walkthrough

B owns the PostgreSQL setup, shared baseline migration, connection pool,
readiness check, asset APIs, collection queries, feature tests and CI setup.
Employee and assignment/report APIs remain A's and C's work. The shared server
shell exists so this module can run and be reviewed before those APIs arrive.

## Request flow and files

```text
HTTP request -> routes -> validation -> repository -> PostgreSQL
                  -> JSON response or shared AppError
```

| File | Responsibility |
| --- | --- |
| Cargo.toml / Cargo.lock | Compatible dependencies and the application's locked resolution |
| compose.yaml / .env.example | Dedicated PostgreSQL and example local configuration |
| migrations/202611120001_create_assetdesk_schema.sql | Agreed employee, asset and assignment schema and constraints |
| src/config.rs | Required database URL, pool size and listening address validation |
| src/db.rs | SQLx connection pool and embedded migrations |
| src/state.rs | A clonable state containing PgPool |
| src/main.rs | Load configuration, connect, migrate, serve and shut down gracefully |
| src/app.rs | Assemble asset routes, liveness/readiness, JSON fallbacks and request tracing |
| src/error.rs | Shared HTTP errors without raw SQL or credentials |
| src/telemetry.rs | Structured JSON logging configured by RUST_LOG |
| src/openapi.rs | Assemble implemented OpenAPI definitions |
| src/features/assets/model.rs | Asset and operational-status types |
| src/features/assets/dto.rs | Request validation, PATCH behavior and query options |
| src/features/assets/repository.rs | Bound queries, derived availability and transactional updates |
| src/features/assets/routes.rs | HTTP extraction, response codes, Location and OpenAPI operations |
| tests/assets.rs / tests/common | Real-PostgreSQL HTTP, schema and concurrency tests with isolated fixtures |
| .github/workflows/ci.yml | Formatting, Clippy and tests with a PostgreSQL service |

## Implemented endpoints

| Method | Path | Result |
| --- | --- | --- |
| POST | /api/v1/assets | Register an asset; 201 with its Location and JSON |
| GET | /api/v1/assets | Return items, limit and offset |
| GET | /api/v1/assets/{id} | Return asset details and derived availability |
| PATCH | /api/v1/assets/{id} | Update descriptive fields or operational status |
| GET | /health | Process liveness, independent of PostgreSQL |
| GET | /ready | SELECT 1 with a timeout; 200 or 503 |
| GET | /api-docs/openapi.json | Specification for implemented routes |

### Register and retrieve

```bash
xh POST http://127.0.0.1:3000/api/v1/assets asset_tag=" lap-021 " name="Development laptop" category=laptop
xh GET http://127.0.0.1:3000/api/v1/assets
```

The returned tag is `LAP-021`. Names/categories are trimmed. The default status
is `in_service`; maintenance/retired assets can also be registered explicitly.
An existing canonical tag produces `409` with `duplicate_asset_tag`.

Use the returned UUID in subsequent requests:

```bash
xh GET http://127.0.0.1:3000/api/v1/assets/ASSET_UUID
xh PATCH http://127.0.0.1:3000/api/v1/assets/ASSET_UUID name="QA laptop"
xh PATCH http://127.0.0.1:3000/api/v1/assets/ASSET_UUID operational_status=maintenance
```

Replace `ASSET_UUID` with that actual identifier. Tags are immutable. Omitted
PATCH fields retain their values; explicit null, unknown fields, attempted tag
changes, and an empty update are rejected. Length limits count characters.

### Search and collections

```bash
xh GET http://127.0.0.1:3000/api/v1/assets category==laptop operational_status==in_service availability==available
xh GET http://127.0.0.1:3000/api/v1/assets q==LAP sort==asset_tag order==asc limit==10 offset==0
```

- `q` is a case-insensitive literal substring of name/tag, with a 120-character limit.
- `category` is an exact match; `operational_status` accepts in_service, maintenance or retired.
- `availability` accepts available, assigned or unavailable.
- `sort` accepts asset_tag, name or created_at; default created_at.
- `order` accepts asc or desc; default desc. The ID breaks ties in the same direction.
- `limit` defaults to 20 and must be 1..100; offset defaults to 0 and must be nonnegative.

Only fixed enum-selected SQL identifiers/directions are interpolated. Query
values are bound. Search escapes wildcard characters so a literal percent or
underscore does not unexpectedly match everything.

Availability is calculated from operational status and outstanding assignments.
Returned history neither hides availability nor duplicates assets in collections.

### Transactional status updates

Changing an asset into maintenance/retired first locks its row, then checks for
outstanding assignments in the same transaction. If assigned, the entire PATCH
returns `409 asset_already_assigned`, including rollback of any accompanying
name/category changes. Descriptive changes to an assigned in-service asset are
permitted.

C must acquire the same asset lock before issuing equipment. See the shared
[transaction contract](B_DATABASE.md#transaction-contract-for-a-and-c). The
database's partial unique index independently prevents two outstanding
assignments for one device.

## Verification and team integration

Unit tests cover configuration, normalization, character limits, PATCH
omission/null semantics, pagination and literal search escaping. Integration
tests exercise the HTTP router against isolated PostgreSQL databases.

The status-change race test holds an asset row lock while inserting an
assignment, starts the competing PATCH, observes PostgreSQL waiting on the lock,
then commits the issue. The PATCH must recheck custody and reject the update.
SQL fixtures stand in for A/C APIs while those features are unfinished.

A can add employee routes to app.rs and merge their OpenAPI definitions in
openapi.rs. C can add assignment/report routes using the existing pool and
baseline tables. Neither needs to recreate the schema. Append new migrations
after the baseline is shared; do not edit an already applied migration.

Future assignment operations remain responsible for employee activity checks,
versioned returns and damaged-return maintenance changes. B does not simulate
those endpoints or silently skip database tests.
