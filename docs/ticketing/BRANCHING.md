# Team branching and integration workflow

Recorded 2026-11-17. This describes the chosen recommendation and the branches
created for this milestone. It does not certify repository-rule enforcement,
teammate review, merged PRs or working application code.

## Recommended strategy

Use a lightweight feature-branch workflow with develop as the integration base
and main as the reviewed release branch. Keep the three service branches short
for this milestone and send small, focused PRs into develop.

This team adaptation uses the branch/review principles in
[GitHub flow](https://docs.github.com/en/get-started/using-github/github-flow).
Its extra develop branch preserves the workflow already selected for this repo.

```text
develop -> feat/support-users ------> reviewed PR -> develop
develop -> feat/support-categories -> reviewed PR -> develop
develop -> feat/support-tickets ----> reviewed PR -> develop

develop -> reviewed release PR -> main
```

## Created branches and current state

| Owner | Created locally and on origin | Upstream |
| --- | --- | --- |
| A | feat/support-users | origin/feat/support-users |
| B: Shreyas | feat/support-categories | origin/feat/support-categories |
| C | feat/support-tickets | origin/feat/support-tickets |

All three were created from the same verified origin/develop commit, b56c317,
and published in one atomic normal push. None includes the removed application
source. They initially contain the existing develop documentation baseline,
not the new planning files or a Rust application.

The Markdown-only plans are prepared separately on docs/support-ticketing-plan
for review into develop. Feature PRs do not exist yet: an unchanged branch has
no implementation diff to review. This work does not directly push to or merge
main/develop.

## Immediate integration dependencies

1. Review the planning/cleanup PR into develop so everyone receives the same
   project scope, team tasks and contracts.
2. Each member fetches and normally merges origin/develop into their branch.
3. B coordinates the fresh T-X00 Rust/config/DB/error/CI foundation in a small
   first PR; A/C review it. A and C can work on agreed DTOs, validation and pure
   rule tests while that PR is prepared.
4. After the foundation merges, all branches sync again. Merge migrations in
   dependency order: users, categories/membership, then tickets/events.
5. Do not complete A/B assignment-removal or role/deactivation guards before
   C's tickets table exists and those rules have real integration tests.

No new migration should run on retained PostgreSQL data until the team records
the database baseline/compatibility choice. Prefer a separate empty development
database; retain the existing data and volume.

## Getting onto your branch

In a clean clone, A uses:

```bash
git fetch origin
git switch --track origin/feat/support-users
```

B uses:

```bash
git fetch origin
git switch --track origin/feat/support-categories
```

C uses:

```bash
git fetch origin
git switch --track origin/feat/support-tickets
```

If the local branch already exists, use git switch followed by its local name
without --track. Do not switch a checkout containing someone else's uncommitted
files. The original worktree remains separate and is not a clean team clone.

## Daily work and synchronization

From your own feature branch, after committing any current work:

```bash
git fetch origin
git merge origin/develop
```

Resolve any conflicts, run the checks relevant to the changed code, then commit
and push normally. Merging an updated develop avoids rewriting already shared
commits. Do not use rebase followed by a force push in this team workflow.

Before the Rust foundation exists, use documentation checks. The current
repository contains Markdown only; recreate configuration and Compose in T-X00.
After the foundation exists, use fmt, Clippy and real-PostgreSQL tests as
appropriate. There is no Cargo project or active app CI in this baseline.

Example A commit/PR flow after implementation exists:

```bash
git add -- src/features/users tests/users.rs
git commit -m "feat(users): add user creation and retrieval"
git push
```

Include the actual migration/docs paths in the explicit staged list. Open a PR
with base develop, the correct feature head, story IDs, acceptance evidence,
schema/API effects and remaining scope. Prefer a draft PR for early feedback.
Another teammate reviews before merging. Never stage .env or unrelated files.

## Merge and release policy

Repository inspection on 2026-11-17 confirmed merge commits are enabled. This
recommendation does not change the repository's available merge methods.

The owner separately requested a documentation-only main README update. Prepare
that bootstrap update as a normal PR against main, containing Markdown files
only and preserving the ordinary implementation workflow below. Do not bypass
any repository rule or include application code in that documentation update.

- Prefer merge commits for the current service PRs, preserving ancestry while
  a service branch has further milestone work. This avoids a shared-history
  rewrite. Enable/confirm the repository's merge method before using it.
- Merge dependencies into develop before dependent code. Do not pull private
  unpublished code from another developer's working directory.
- After a feature PR merges, normally merge updated origin/develop into the
  remaining feature branches. Rerun affected integration checks.
- Delete a service branch after its milestone work is complete. New unrelated
  work gets a new branch from updated develop.
- Release through a reviewed develop -> main PR with exact-commit checks,
  acceptance/demo evidence and known limitations. Create a version tag only
  when there is an actual accepted release.

## Recommended repository rules

[GitHub branch protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
can require review/checks and block force pushes. Recommended policy:

| Branch scope | Policy |
| --- | --- |
| main and develop | PR required, at least one teammate approval, conversations resolved, relevant checks once present, no direct pushes, no deletion and no administrator bypass |
| All shared branches | Block force pushes; feature branches still allow normal pushes |
| Completed feature branches | Permit deliberate deletion after the work is integrated |

Use compatible rulesets if combining a global force-push restriction with more
specific main/develop rules. Do not require a nonexistent CI check: add the
required check after its workflow exists and has a known successful run.
No repository rules/settings were modified by documenting this recommendation.

## Suggested first PRs

| PR | Owner | Review focus |
| --- | --- | --- |
| Markdown-only plans | Planning branch; team review | Current scope, removed configuration/legacy files and shared instructions |
| T-X00 shared foundation | B; A/C review | Build/startup, DB baseline, errors, health/readiness, CI and test setup |
| T-A01 user create/get/list and migration | A | Normalization, uniqueness, DTOs and real-DB tests |
| T-B01 category create/get/list and migration | B | Category contract, unique names and real-DB tests |
| T-C01/02 ticket create/get/list and schema | C | Active requester/category, initial state/version, collection behavior |
| Eligibility, claim and lifecycle slices | A/B/C according to ownership | Shared locking, guarded configuration changes, versions and atomic events |

See [detailed team tasks](TEAM_TASKS.md) and [delivery records](DELIVERY.md).
