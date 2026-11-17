# Ticketing decisions and cleanup record

Recorded 2026-11-17. This records project direction and local cleanup, not
completed ticketing implementation, business acceptance or a release.

## CR-004: internal employee support

The owner selected an internal support ticketing system with Infra and HR
categories, user management and category management. The final name remains
undecided. Jira Service Management (Atlassian), ServiceNow, Zendesk and the
earlier ServiceDesk Plus reference inform selected workflow patterns.

Ownership: A users/access, B categories/membership/database coordination,
C ticket lifecycle and history. The selected short-notice prototype has five
business tables. Comments, reassignment, advanced queries and authentication
remain follow-up work; company-use access enforcement is still required.

See the [short-notice design](SHORT_NOTICE_DESIGN.md), [requirements](REQUIREMENTS.md)
and [delivery plan](DELIVERY.md) for scope and acceptance.

## CR-005: remove superseded implementation

The owner requested complete removal of old code and unnecessary files from
the active AssetDesk-B project folder. Removed:

- Old Rust source, including the inventory API and its shared runtime files.
- Cargo manifest/lockfile, inventory tests and old migration source.
- Device examples and the old Cargo-based GitHub Actions workflow.
- Superseded inventory/B guides and inventory-specific Agile documents.
- Generated target build cache.

Kept the current ticketing plans, README/index, compose.yaml, ignored local
.env, .env.example, .gitignore and generic PR/issue templates.

Git metadata/history, main/develop, PostgreSQL containers/data/volumes and other
repositories/checkouts were not deleted. The separate ReleaseReady checkout
holds shared Git metadata and uncommitted drafts and remains intact.

A private backup of removed source/configuration/documents was saved outside
the project at /tmp/assetdesk-legacy-cleanup.GGOMvm/old-implementation.tar.gz.
The generated build cache was excluded. This temporary backup is not a durable
archive or a database backup; prior committed code also remains in Git history.

## Baseline after CR-005

This is a planning-and-database-setup baseline. No current Rust package,
application executable, API routes, generated OpenAPI, app CI or tests exist.
Database-only commands are in the root README. Future cargo/API commands in
the plans apply only after implementing a new foundation.

T-X00 now means bootstrap and review a fresh foundation into develop. Before
running new migrations, decide whether to use a fresh database or a migration
path compatible with existing data and SQLx migration history. No database
reset or schema deletion was performed by this cleanup.

## CR-006: service ownership and team branches

The owner requested clearer A/B/C assignments, branches for all three and a
recommended branching strategy. [TEAM_TASKS.md](TEAM_TASKS.md) defines APIs,
files/functions, dependencies, first PRs and acceptance evidence for each owner.

Created locally and on origin from origin/develop at b56c317:
feat/support-users (A), feat/support-categories (B: Shreyas) and
feat/support-tickets (C). Each tracks its matching remote branch. They were
published together with an atomic normal push; no removed source was restored.

The selected recommendation keeps main for releases, develop for integration
and reviewed feature PRs for implementation. The clean planning/setup baseline
is prepared for review separately; teammates sync after it and the new shared
foundation reach develop. [BRANCHING.md](BRANCHING.md) records the procedure.
Branch creation does not merge protected branches or alter repository rules.

## CR-007: Markdown-only repository and main overview

The owner clarified that the entire active project should retain Markdown files
only and requested the updated project README on main. Removed compose.yaml,
.env.example, .gitignore, the ignored local .env, local editor settings and
regenerated build output.
Current project content is README/documentation and Markdown review/issue
templates. Git metadata/history and external database data/volumes are retained.

The removed configuration, including private local environment settings, is
backed up outside the project at
/tmp/assetdesk-markdown-cleanup.k5Ewmk/configuration.tar.gz with private access.
It is not committed or published and is not a database backup.
Local editor settings are stored separately in editor-settings.tar.gz in the
same private backup directory.

The README now presents the ticketing workflow, five-table milestone, A/B/C
ownership/branches, stack, P1–P6 learning and linked documentation without
claiming a runnable application or existing setup files. All runtime,
configuration, migration, test and CI artifacts are future foundation work.

The explicit documentation update for main uses a separate normal PR from a
main-based documentation branch. Application feature work continues through
develop; no force push, rule bypass or database reset is part of this request.
