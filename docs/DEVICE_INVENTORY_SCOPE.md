# Company IT infrastructure and electronics inventory

Scope change CR-003, requested 2026-11-17. Final project name is pending the
owner's decision. AssetDesk remains the technical identifier for now.

## Product purpose

Maintain a register of basic electronic devices and accessories owned by the
company: identify each item, find it by type, record operational status and track
responsibility/custody where applicable. Inventory is the primary purpose;
employee issue/return is one workflow for portable equipment.

The scope is physical IT/electronic inventory. Network monitoring, remote device
control, Wi-Fi credentials, automatic discovery and repair-ticket automation are
not introduced by the phrase IT infrastructure management.

## Initial device types

| Physical item | Suggested category | Example tag | Typical use; verify actual deployment |
| --- | --- | --- | --- |
| TV | tv | TV-001 | Meeting-room/shared or installed equipment |
| TV remote | tv_remote | REM-001 | Shared accessory associated with a room/TV |
| Laptop | laptop | LAP-001 | Employee-issued or pooled portable equipment |
| HDMI connector/adapter | hdmi_connector | HDMI-001 | Portable/shared accessory |
| Wi-Fi device | wifi_device | WIFI-001 | Installed access point, router or repeater |
| Laptop charger | laptop_charger | CHG-001 | Issued accessory or shared spare |
| UPS | ups | UPS-001 | Installed power-protection equipment |

Category does not decide custody automatically: a TV can be portable and a charger
fixed to a desk. Record actual deployment explicitly when that feature lands.
Each physical unit gets its own asset_tag, including accessories without serial
numbers. Two chargers are two tagged records, not quantity=2 in one record.

## What works in the current B API

Category is bounded nonempty text, not a closed Rust enum/database list. All seven
suggested values work with the existing create/filter contract. New categories
such as usb_hub or projector require no code/schema change. Use consistent spelling.

Supported fields: asset_tag, name, category and operational_status, plus generated
ID/timestamps and derived available. Status is in_service, maintenance or retired;
it is an inventory record, not evidence that hardware is online.

```bash
xh POST http://127.0.0.1:3000/api/v1/assets "asset_tag=TV-001" "name=Meeting room TV" "category=tv"
xh GET http://127.0.0.1:3000/api/v1/assets category==tv
```

[examples/devices.json](../examples/devices.json) contains seven synthetic POST
bodies. They are not actual company inventory and are not automatically imported.
Registering a tag already present returns 409.

## Next inventory increment: US-B05

These are proposed additions, not current JSON fields/endpoints:

- Location: meeting room, IT cupboard, reception or desk.
- Explicit usage/deployment: assignable portable, shared or installed.
- Distinction between operational/unassigned and eligible for employee issue.
- Optional serial number/brand/model only if selected as useful by the team.

Current available means in_service with no outstanding employee assignment. It
does not establish that an installed UPS/TV may be borrowed. Before C offers issue
operations for all device types, B/C must integrate deployment rules and tests.
Installed devices use location/responsibility tracking; portable items use issue/return.

Optional later work: parent-device/accessory relationships (TV -> remote,
laptop -> charger), location movement and structured maintenance history.
They require explicit backlog selection, data design and tests. Do not send
location/usage/parent fields to the current API: unknown fields are rejected.

## Ownership and compatibility

- A: employees, accounts, admin/user access and guards.
- B: device/accessory inventory, category convention, DB and planned location/deployment.
- C: portable allocations/returns/history/reports; coordinate shared/installed
  deployment history after the location model is agreed.

Preserve /api/v1/assets for compatibility. Final branding and repository/folder/
crate/database/container identifiers are separate actions after the owner chooses
a name. Do not recreate data/volumes as a cosmetic rebrand.
