# AssetDesk — Member C Checklist

Status: implementation not started.
Contract reference: origin/docs/support-ticketing-plan

## Domain
- [ ] Coordinate shared TicketType with B.
- [ ] Define Priority and TicketStatus.
- [ ] Implement DTO validation and pure lifecycle rules.
- [ ] Add domain unit tests.

## Database
- [ ] Agree migration order and shared eligibility lock with A/B.
- [ ] Create tickets, ticket_events and SR/INC sequences.
- [ ] Enforce constraints and one event per ticket version.

## API
- [ ] Create tickets with transactional events.
- [ ] Implement scoped get/list with filtering and pagination.
- [ ] Implement versioned manager review and assignment.
- [ ] Implement start, resolve, requester close and reopen.
- [ ] Implement scoped events and own work history.

## Verification
- [ ] Test eligibility, visibility and resolution requirements.
- [ ] Test concurrent reviews, stale versions and rollback.
- [ ] Test reopening resets and retained history.
- [ ] Add OpenAPI and run instructions.
- [ ] Pass checks and obtain teammate review.

## Dependencies
- A: users, roles and shared actor interface.
- B: application foundation, categories and membership.
- Reviewed shared foundation and ordered migrations.
