# Product brief

Status: documented scope for team review. Detailed acceptance lives in the
requirements/backlog; implementation and acceptance are tracked separately.

## Problem and product goal

IT needs to know who holds each company device, whether it can be issued and
what an employee still needs to return. AssetDesk provides a REST API that
records custody, prevents conflicting issues and derives equipment clearance.

The complete demonstration is: create employee/account and asset, authenticate,
issue equipment, explain outstanding custody, reject conflicting actions,
record return and show clearance with retained history.

Expected benefits are visibility, consistent handovers and less manual checking.
No financial savings, adoption level, production scale or performance claim has
been measured. Equipment clearance concerns recorded equipment only, not payroll,
HR authorization or physical device verification.

## Stakeholders and ownership

| Role | Responsibility / known identity |
| --- | --- |
| Product/business acceptance reviewer | Prioritize value and accept the demonstration; company representative to be confirmed |
| A: user/access service | Employee lifecycle, accounts/roles, authentication, shared guards and API assembly |
| B: Shreyas, inventory/database | Asset API, schema/pool coordination, readiness, CI/setup and database handoff |
| C: custody/clearance service | Issue/return, history, version/concurrency rules, reports and end-to-end coordination |
| Feature reviewer | A different teammate; proposed rotation A reviews B, B reviews C, C reviews A |
| Operator | Own environment, backups and recovery; to be named before deployment |

Everyone owns their feature tests, validation and OpenAPI definitions. This is
an ownership split, not three separate deployment teams.

## Product boundary

In scope: individually tagged assets, employees/accounts, admin/user access,
issue/return history, overdue reporting, derived clearance, safe concurrent
updates and documented/tested HTTP APIs.

Deferred: frontend, email notifications, approval workflows, purchasing,
depreciation, payroll, device agents/wiping, attachments, QR scanning, SSO/MFA,
refresh-token/email-reset services. Deadline editing/export remain optional.

Original planning used three contributors and three days. Added authentication
changed that scope. Agree remaining capacity and delivery dates during planning;
the old estimate is not evidence of a current delivery commitment.

## Success and acceptance

- A teammate can start from a clean checkout and run the documented increment.
- Its selected story criteria and negative/concurrency cases pass against PostgreSQL.
- Roles and employee ownership are enforced once the access service is integrated.
- The exact candidate has review/CI evidence and an honest remaining-work list.
- The business reviewer accepts a demonstrated result, with limitations recorded.

Documentation prepared after some coding is a retrospective record of design
and scope. Do not invent earlier kickoff decisions, meeting dates or sign-offs.
