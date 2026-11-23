use super::model::{ReviewState, TicketStatus, check_version};
use super::validation::{ValidationError, validate_resolution};
use crate::features::categories::model::TicketType;

#[derive(Debug, PartialEq, Eq)]
pub enum SubmitResolutionError {
    VersionConflict,
    InvalidStatus,
    IncompleteReview,
    NotAssignedHandler,
    InvalidText(ValidationError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmittedResolution<'a> {
    pub status: TicketStatus,
    pub resolution: &'a str,
}

pub fn submit_resolution<'a, Id: Eq>(
    ticket: &ReviewState,
    expected_version: i64,
    assignee: Option<&Id>,
    actor: &Id,
    resolution: &'a str,
) -> Result<SubmittedResolution<'a>, SubmitResolutionError> {
    check_version(ticket.version, expected_version)
        .map_err(|_| SubmitResolutionError::VersionConflict)?;
    let status = TicketStatus::AwaitingEmployeeConfirmation;
    if !ticket.status.can_transition_to(status) {
        return Err(SubmitResolutionError::InvalidStatus);
    }
    if !(ticket.has_reviewer && ticket.has_review_timestamp && ticket.has_assignee) {
        return Err(SubmitResolutionError::IncompleteReview);
    }
    if assignee != Some(actor) {
        return Err(SubmitResolutionError::NotAssignedHandler);
    }
    validate_resolution(resolution).map_err(SubmitResolutionError::InvalidText)?;
    Ok(SubmittedResolution {
        status,
        resolution: resolution.trim(),
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmResolutionError {
    NotRequester,
    VersionConflict,
    InvalidStatus,
}

/// The requester ID comes from the stored ticket and the actor ID from the actor context.
pub fn confirm_resolution<Id: Eq>(
    ticket: &ReviewState,
    expected_version: i64,
    requester: &Id,
    actor: &Id,
) -> Result<TicketStatus, ConfirmResolutionError> {
    if actor != requester {
        return Err(ConfirmResolutionError::NotRequester);
    }
    check_version(ticket.version, expected_version)
        .map_err(|_| ConfirmResolutionError::VersionConflict)?;
    if !ticket.status.can_transition_to(TicketStatus::Resolved) {
        return Err(ConfirmResolutionError::InvalidStatus);
    }
    Ok(TicketStatus::Resolved)
}

#[cfg(test)]
mod tests {
    use super::super::model::Priority;
    use super::*;

    #[test]
    fn submission_requires_current_handler_version_status_and_valid_text() {
        let mut ticket = ReviewState {
            priority: Priority::P2,
            version: 3,
            status: TicketStatus::InProgress,
            has_assignee: true,
            has_reviewer: true,
            has_review_timestamp: true,
        };
        let submit = |ticket: &ReviewState, version, actor, text: &'static str| {
            submit_resolution(ticket, version, Some(&20), &actor, text)
        };
        assert_eq!(
            submit(&ticket, 3, 20, "  Restored VPN  "),
            Ok(SubmittedResolution {
                status: TicketStatus::AwaitingEmployeeConfirmation,
                resolution: "Restored VPN",
            })
        );
        assert_eq!(
            submit(&ticket, 3, 21, "Fixed"),
            Err(SubmitResolutionError::NotAssignedHandler)
        );
        assert_eq!(
            submit(&ticket, 2, 20, "Fixed"),
            Err(SubmitResolutionError::VersionConflict)
        );
        assert_eq!(
            submit(&ticket, 3, 20, "  "),
            Err(SubmitResolutionError::InvalidText(
                ValidationError::InvalidResolution
            ))
        );
        for status in [
            TicketStatus::Open,
            TicketStatus::AwaitingEmployeeConfirmation,
            TicketStatus::Resolved,
        ] {
            ticket.status = status;
            assert_eq!(
                submit(&ticket, 3, 20, "Fixed"),
                Err(SubmitResolutionError::InvalidStatus)
            );
        }
    }

    #[test]
    fn only_requester_can_confirm_a_pending_resolution_at_the_current_version() {
        let mut ticket = ReviewState {
            priority: Priority::P1,
            version: 4,
            status: TicketStatus::AwaitingEmployeeConfirmation,
            has_assignee: true,
            has_reviewer: true,
            has_review_timestamp: true,
        };
        assert_eq!(
            confirm_resolution(&ticket, 4, &10, &20),
            Err(ConfirmResolutionError::NotRequester)
        );
        assert_eq!(
            confirm_resolution(&ticket, 3, &10, &10),
            Err(ConfirmResolutionError::VersionConflict)
        );
        ticket.status = confirm_resolution(&ticket, 4, &10, &10).unwrap();
        assert_eq!(ticket.status, TicketStatus::Resolved);
        for status in [
            TicketStatus::Resolved,
            TicketStatus::Open,
            TicketStatus::InProgress,
        ] {
            ticket.status = status;
            assert_eq!(
                confirm_resolution(&ticket, 4, &10, &10),
                Err(ConfirmResolutionError::InvalidStatus)
            );
        }
    }
}

/// Category fields needed by ticket creation, supplied by the category lookup.
pub struct CreationCategory {
    pub name: String,
    pub ticket_type: TicketType,
    pub is_active: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreationError {
    CategoryLookupFailed,
    CategoryNotFound,
    CategoryInactive,
    CategoryTypeMismatch,
    Input(super::dto::CreateInputError),
}

/// Validate a proposed ticket against the current category record.
pub async fn validate_creation_in_db(
    pool: &sqlx::PgPool,
    input: &super::dto::CreateTicketInput<uuid::Uuid>,
) -> Result<(), CreationError> {
    let category = crate::features::categories::repository::find_category(pool, input.category_id)
        .await
        .map_err(|_| CreationError::CategoryLookupFailed)?;
    validate_creation(input, |_| {
        category.map(|c| CreationCategory {
            name: c.name,
            ticket_type: c.ticket_type,
            is_active: c.is_active,
        })
    })
}

pub fn initialize_ticket<Id>(
    input: &super::dto::CreateTicketInput<Id>,
    find_category: impl FnOnce(&Id) -> Option<CreationCategory>,
) -> Result<ReviewState, CreationError> {
    validate_creation(input, find_category)?;
    Ok(ReviewState {
        priority: input.priority,
        version: 1,
        status: TicketStatus::Open,
        has_assignee: false,
        has_reviewer: false,
        has_review_timestamp: false,
    })
}

pub fn validate_creation<Id>(
    input: &super::dto::CreateTicketInput<Id>,
    find_category: impl FnOnce(&Id) -> Option<CreationCategory>,
) -> Result<(), CreationError> {
    let category = find_category(&input.category_id).ok_or(CreationError::CategoryNotFound)?;
    if !category.is_active {
        return Err(CreationError::CategoryInactive);
    }
    input
        .validate(&category.name)
        .map_err(CreationError::Input)?;
    if category.ticket_type != input.ticket_type {
        return Err(CreationError::CategoryTypeMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod creation_tests {
    use super::super::{
        dto::{CreateInputError, CreateTicketInput},
        model::Priority,
    };
    use super::*;

    #[test]
    fn creation_checks_the_category_selected_by_id() {
        let input = CreateTicketInput {
            category_id: 42,
            ticket_type: TicketType::Incident,
            priority: Priority::P3,
            subject: "VPN".into(),
            description: "Connection drops".into(),
            custom_concern: None,
        };
        let check = |id, active, kind: TicketType, name: &str| {
            validate_creation(&input, |requested| {
                (*requested == id).then(|| CreationCategory {
                    name: name.into(),
                    ticket_type: kind,
                    is_active: active,
                })
            })
        };
        assert_eq!(check(42, true, TicketType::Incident, "Network"), Ok(()));
        assert_eq!(
            check(41, true, TicketType::Incident, "Network"),
            Err(CreationError::CategoryNotFound)
        );
        assert_eq!(
            check(42, false, TicketType::Incident, "Network"),
            Err(CreationError::CategoryInactive)
        );
        assert_eq!(
            check(42, true, TicketType::ServiceRequest, "Network"),
            Err(CreationError::CategoryTypeMismatch)
        );
        assert_eq!(
            check(42, true, TicketType::Incident, "Others"),
            Err(CreationError::Input(CreateInputError::MissingCustomConcern))
        );
    }

    #[test]
    fn newly_created_tickets_are_pending_review_at_version_one() {
        for priority in [Priority::P1, Priority::P2, Priority::P3, Priority::P4] {
            let input = CreateTicketInput {
                category_id: 7,
                ticket_type: TicketType::ServiceRequest,
                priority,
                subject: "Software".into(),
                description: "Install the editor".into(),
                custom_concern: None,
            };
            assert!(matches!(
                initialize_ticket(&input, |_| None),
                Err(CreationError::CategoryNotFound)
            ));
            let state = initialize_ticket(&input, |_| {
                Some(CreationCategory {
                    name: "Software".into(),
                    ticket_type: TicketType::ServiceRequest,
                    is_active: true,
                })
            })
            .unwrap();
            assert_eq!(state.priority, priority);
            assert_eq!(state.version, 1);
            assert!(super::super::model::is_pending_review(
                state.status,
                state.has_assignee,
                state.has_reviewer,
                state.has_review_timestamp
            ));
        }
    }
}

/// requester_id must come from the trusted actor context.
pub async fn requester_ticket_page(
    pool: &sqlx::PgPool,
    requester_id: uuid::Uuid,
    query: super::dto::ListTicketsQuery,
) -> Result<
    crate::features::categories::dto::Page<super::repository::TicketDetails>,
    super::repository::ListTicketsError,
> {
    use super::repository::{TicketFilters, query_requester_tickets};
    let filters = TicketFilters {
        status: query.status,
        priority: query.priority,
        ticket_type: query.ticket_type,
        category_id: query.category_id,
    };
    let items = query_requester_tickets(
        pool,
        requester_id,
        filters,
        Some(query.limit),
        Some(query.offset),
    )
    .await?;
    Ok(crate::features::categories::dto::Page {
        items,
        limit: query.limit,
        offset: query.offset,
    })
}
