use super::model::{ReviewState, TicketStatus, check_version};
use super::validation::{ValidationError, validate_resolution};

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
}
