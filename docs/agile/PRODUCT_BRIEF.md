# Product brief

Status: documented scope for team review. Detailed acceptance lives in the
requirements/backlog; implementation and acceptance are tracked separately.

## Problem and product goal

IT needs a register of company electronic devices and accessories: TVs, remotes,
laptops, HDMI connectors, Wi-Fi devices, laptop chargers and UPS units, with more
types added later. AssetDesk is a working technical name; final branding is pending.

The primary goal is inventory visibility and operational status. Shared/installed
devices need location/deployment tracking; portable equipment may be issued to
employees and returned. The target demonstration includes registering shared
infrastructure and portable accessories, locating devices, changing status and
recording a portable item's handover/return. Location/deployment is planned, not
implemented in B yet. See [device scope](../DEVICE_INVENTORY_SCOPE.md).

Expected benefits are visibility, consistent handovers and less manual checking.
No financial savings, adoption level, production scale or performance claim has
been measured. Equipment clearance concerns recorded equipment only, not payroll,
HR authorization or physical device verification.

## Stakeholders and ownership

| Role | Responsibility / known identity |
| --- | --- |
| Product/business acceptance reviewer | Prioritize value and accept the demonstration; company representative to be confirmed |
| A: user/access service | Employee lifecycle, accounts/roles, authentication, shared guards and API assembly |
| B: Shreyas, inventory/database | Electronic-device/accessory API, categories, planned location/deployment, schema/pool, CI/setup and handoff |
| C: custody/clearance service | Issue/return, history, version/concurrency rules, reports and end-to-end coordination |
| Feature reviewer | A different teammate; proposed rotation A reviews B, B reviews C, C reviews A |
| Operator | Own environment, backups and recovery; to be named before deployment |

Everyone owns their feature tests, validation and OpenAPI definitions. This is
an ownership split, not three separate deployment teams.

## Product boundary

In scope: individually tagged electronic devices/accessories, extensible
categories, operational states, planned location/deployment, accounts/roles and
portable-item custody/history. Clearance remains a portable-custody subfeature.
The current category field supports future types without a code/schema change.

Deferred: frontend, email notifications, approval workflows, purchasing,
depreciation, payroll, device agents/wiping, attachments, QR scanning, SSO/MFA,
refresh-token/email-reset services, network monitoring, automatic discovery and
remote control. Parent/accessory links, movement and maintenance-history workflows
are later optional increments. Deadline editing/export remain optional.

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
