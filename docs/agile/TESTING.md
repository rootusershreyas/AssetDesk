# Test strategy, traceability and acceptance evidence

## Approach

Use unit tests for pure validation/configuration/calculations and real PostgreSQL
for queries, migrations, constraints and transactions. HTTP integration tests
exercise the same application router. Manual xh/jq checks demonstrate the
selected increment; they complement automated regression tests.

Every story needs expected success, validation/failure, authorization and shared
state/concurrency behavior where relevant. Test data is synthetic and databases
isolated. Never point integration tests at production or silently skip an absent DB.

## Requirements-to-test matrix

Existing B tests are in [tests/assets.rs](../../tests/assets.rs). Names below
identify verifiable source functions, not a claim of a new run for changed code.

| Story / requirement | Implementation / evidence | Remaining verification |
| --- | --- | --- |
| US-A01 | A employee contract and shared schema | Full employee API/deactivation/session integration tests pending |
| US-A02/03, NFR-03 | Separate access draft; account/session design | Integrated login/logout/expiry/revocation, role and ownership negatives, last-admin race and password lifecycle on actual candidate |
| US-B01 | creates_normalizes_retrieves_and_rejects_duplicate_tags; validates_create_and_patch_without_losing_omitted_values | Baseline passed; integrated admin guard added/tested by access owner |
| US-B02 | derives_availability_from_current_custody_and_filters_assets; searches_literal_wildcards_and_backslashes; sorts_and_paginates_with_stable_id_tie_breakers | Baseline passed; repeat after changed query/schema integration |
| US-B03, NFR-02 | rejects_assigned_asset_status_changes_atomically; status_change_waits_for_issue_lock_then_rechecks_custody | Baseline passed; add integrated C issue/status race after C lands |
| US-B04, NFR-04/06 | migration_enforces_custody_history_and_return_constraints; readiness_tracks_database_while_health_stays_live | Baseline passed; clean integrated setup/upgrade verification |
| US-B05 | New location/deployment contract, planned | Test declared usage/locations, eligibility, backfill and new/unknown future categories after implementation; current sample JSON validates the existing POST shape only |
| US-C01 | Partial unique index already tested | Actual overlapping issue: one success, one 409, one open row; inactive/maintenance and US-B05 deployment-eligibility cases |
| US-C02 | Row-local return checks in baseline | Version/repeat/damaged return, inactive borrower and atomic rollback tests for C API |
| US-C03 | History/ownership collection contracts | Admin filters/sort/page and user-isolation tests on integrated service |
| US-C04/US-X01 | Reporting/end-to-end design | Empty/missing/no-deadline/returned/boundary-time reports; complete clean-checkout scenario and business acceptance |
| NFR-01/07 | returns_consistent_errors_for_missing_and_malformed_requests; openapi_documents_assets_filters_and_conflict_responses | Contract/security and review/release evidence after integration |
| NFR-05 | B structured logs/liveness; proposed access no-store behavior | Inspect integrated logs/headers for absence of credentials and correct actor-safe diagnostics |

B unit tests sit beside configuration, DTO and literal-search functions.
SQLx test fixtures create employee/assignment rows directly while A/C endpoints
are unfinished. Database fixtures do not prove those missing endpoints work.

## Reproducible checks

With dedicated PostgreSQL running and .env configured:

```bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

CI provisions PostgreSQL and runs formatting, Clippy and tests. On implementation
changes, record the exact commit, commands/results and run URL. A clean CI run
does not replace business acceptance, restore evidence or production approval.
For documentation-only edits, check changed Markdown/link correctness; do not
invent a fresh runtime result or rerun unrelated tests just to inflate evidence.

## Concurrency and safety checks

- Overlap requests on separate connections; sequential duplicate calls are not
  proof of concurrency correctness. Assert outcomes and final stored state.
- B's race test observes a PostgreSQL lock wait, commits a competing issue fixture,
  and verifies the waiting PATCH rechecks custody and rolls back metadata.
- Include malformed/null/unknown inputs, stale versions, invalid transitions,
  wrong/missing identity, forged owner/role and expired/revoked session cases.
- Failed operations leave no partial writes. Returned history must remain.
- Clearance must use consistent rows and explicit time for deterministic boundary tests.

## Evidence ledger

| Observation | Result | Scope |
| --- | --- | --- |
| B commit 2959a4c | Historical 7 unit + 11 PostgreSQL tests, formatting/Clippy and live checks passed | Standalone B baseline, not later draft auth/code |
| [B PR CI](https://github.com/rootusershreyas/AssetDesk/actions/runs/43174568637) | Recorded successful run; PR inspected 2026-11-16 | Exact B baseline |
| Owner run, 2026-11-16 | /ready 200; misspelled /asserts 404 | Connectivity and route fallback; not successful asset creation evidence |
| New documentation baseline | Record link/format checks in its commit/PR | Documentation only |
| Integrated product / release candidate | Pending | No final test/UAT/release success claimed |

## Complete acceptance scenario

1. Clean checkout, configuration, database/migration and secure first-admin setup.
2. Admin registers all initial electronic types; after US-B05 lands, demonstrate
   shared/installed location tracking and portable eligibility. Create employee/
   account; user login accesses only own records.
3. Issue available asset; show outstanding custody and incomplete clearance.
4. Demonstrate conflicting issue/status change and unauthorized access rejection.
5. Return using current version; verify history, availability and clearance.
6. Repeat/stale return rejected; damaged-return path sets maintenance atomically.
7. Record candidate/CI, observed versus expected results, limitations and reviewer.

Use [B's manual checks](../B_ACCEPTANCE.md) for the standalone inventory increment.
Never mark this complete integrated scenario passed while A/C writes are absent.
