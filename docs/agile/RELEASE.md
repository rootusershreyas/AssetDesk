# Release, deployment and handoff runbook

Status: procedure/checklist, not a completed deployment. Production environment,
operator and release version are not yet chosen. B can run independently; full
product acceptance remains pending A/C/access integration.

## Run the current B increment

Prerequisites: Rust with rustfmt/Clippy, Docker Compose, xh and jq. From the repo
root, create .env only if one does not already exist; retain local configuration.

```bash
cp .env.example .env
docker compose up -d --wait db
cargo run --locked
```

PostgreSQL defaults to 127.0.0.1:5434 and the API to 127.0.0.1:3000. Startup
applies embedded migrations before serving. Exported environment settings take
precedence over .env. Follow [B setup](../B_DATABASE.md) for details.

```bash
xh --ignore-stdin GET http://127.0.0.1:3000/health
xh --ignore-stdin GET http://127.0.0.1:3000/ready
xh --ignore-stdin GET http://127.0.0.1:3000/api-docs/openapi.json
```

Use RUST_LOG=info,tower_http=debug when demonstrating structured request logs.
Ctrl+C stops the API gracefully; docker compose stop db stops PostgreSQL while
retaining data. Routine shutdown must not delete the volume.

These are current B commands. The standalone B increment has no login guard.
Before an integrated release, A must add verified first-admin bootstrap/login,
session and operator recovery instructions; update this runbook with commands
that exist in that candidate. Do not advertise unfinished endpoints as runnable.

## Candidate release checklist

- [ ] Exact candidate commit, included stories/change IDs and release scope recorded.
- [ ] Relevant code reviewed; full candidate compiles and passes fmt/Clippy/tests/CI.
- [ ] All selected acceptance criteria and complete integration demo pass.
- [ ] Admin/user and employee-ownership restrictions tested on every management/self route.
- [ ] No unresolved release-blocking bugs; remaining limitations explicitly accepted/deferred.
- [ ] Fresh setup/migration/upgrade, sanitized logs and API documentation verified.
- [ ] Environment, operator, protected configuration, backup and restore evidence recorded.
- [ ] Schema/application compatibility and recovery procedure reviewed.
- [ ] Business reviewer accepts the demonstration; teammates review release PR.
- [ ] develop -> main release PR accepted; actual tag/deployment outcome recorded separately.

Unchecked boxes are intentional: no final release acceptance, production rollout
or restore exercise is claimed by creating the documents.

## Backup and restore rehearsal

These examples target only the dedicated local Compose database. Dumps contain
personal data and password hashes after access integration. Restrict permissions,
keep backups outside Git, and confirm each command succeeds.

```bash
mkdir -p ../assetdesk-backups
umask 077
assetdesk_backup=$(mktemp ../assetdesk-backups/assetdesk-XXXXXXXX.dump)
docker compose exec -T db pg_dump -U assetdesk -d assetdesk -Fc > "$assetdesk_backup"
```

Record the path and candidate/migration versions. Protect and store the backup
according to the chosen environment's retention/recovery policy. A nonempty file
is not evidence that recovery works. Restore into a separate new database:

```bash
docker compose exec -T db createdb -U assetdesk -T template0 assetdesk_restore_check
docker compose exec -T db pg_restore -U assetdesk -d assetdesk_restore_check --exit-on-error < "$assetdesk_backup"
docker compose exec -T db psql -U assetdesk -d assetdesk_restore_check -c '\dt'
```

Use a fresh target name if it exists. Compare expected record counts, migration
versions and representative custody records, then smoke-test a separately
configured verification instance. /ready proves connectivity, not restored
business-data correctness. Record the rehearsal; never casually overwrite the
working database to test a backup.

## Deployment and recovery

For a real environment, agree target, HTTPS, secret management, limited DB role,
request throttling, operator access, logging and backup/recovery objectives.
Those choices are environment-specific and not implemented by local Compose.

Before deployment, back up, review migration compatibility, apply the candidate,
check readiness and smoke-test the accepted workflow. Record outcome. If it
fails, stop acceptance and triage the exact failure. Roll back the application
only if compatible with the schema; otherwise use a reviewed forward-fix or a
coordinated restore with explicit downtime/data-loss assessment. Do not edit
applied migrations or force-push Git history as a recovery procedure.

## Release notes and acceptance record

```text
Release/version (chosen by team), date, candidate commit and environment:
Included accepted story IDs and changes:
Test/CI/review/UAT evidence:
Migration/configuration differences and operator instructions:
Backup path/restore evidence and recovery decision:
Known issues/deferred stories and business impact:
Business acceptance reviewer/date/result:
Technical reviewer/date/result:
Deployment/tag outcome, or reason deployment was withheld:
Handoff owner and maintenance contact:
```

No production tag or acceptance signature is prefilled. After release, use
[maintenance and changes](MAINTENANCE.md) to track incidents and next increments.
