#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Open,
    InProgress,
    AwaitingEmployeeConfirmation,
    Resolved,
}
impl TicketStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::InProgress)
                | (Self::InProgress, Self::AwaitingEmployeeConfirmation)
                | (Self::AwaitingEmployeeConfirmation, Self::Resolved)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TicketStatus;

    #[test]
    fn only_documented_transitions_are_allowed() {
        use TicketStatus::*;

        let states = [Open, InProgress, AwaitingEmployeeConfirmation, Resolved];
        let allowed = [
            (Open, InProgress),
            (InProgress, AwaitingEmployeeConfirmation),
            (AwaitingEmployeeConfirmation, Resolved),
        ];

        for from in states {
            for to in states {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "unexpected transition decision: {from:?} -> {to:?}"
                );
            }
        }
    }
}

pub fn is_pending_review(
    status: TicketStatus,
    has_assignee: bool,
    has_reviewer: bool,
    has_review_timestamp: bool,
) -> bool {
    status == TicketStatus::Open
        && !has_assignee
        && !has_reviewer
        && !has_review_timestamp
}

#[cfg(test)]
mod pending_review_tests {
    use super::*;

    #[test]
    fn pending_review_requires_open_and_cleared_assignment_and_review() {
        use TicketStatus::*;
        for status in [Open, InProgress, AwaitingEmployeeConfirmation, Resolved] {
            for has_assignee in [false, true] {
                for has_reviewer in [false, true] {
                    for has_review_timestamp in [false, true] {
                        let expected = (status, has_assignee, has_reviewer, has_review_timestamp)
                            == (Open, false, false, false);
                        assert_eq!(
                            is_pending_review(status, has_assignee, has_reviewer, has_review_timestamp),
                            expected
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VersionConflict;

pub fn check_version(current: i64, expected: i64) -> Result<(), VersionConflict> {
    if current == expected {
        Ok(())
    } else {
        Err(VersionConflict)
    }
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn review_requires_the_current_version() {
        assert_eq!(check_version(2, 2), Ok(()));
        assert_eq!(check_version(2, 1), Err(VersionConflict));
        assert_eq!(check_version(2, 3), Err(VersionConflict));
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentError {
    P1RequiresReviewingManager,
    AssigneeRequired,
    IneligibleMember,
}

pub fn select_assignee<Id: Eq>(
    priority: Priority,
    manager_id: Id,
    requested_id: Option<Id>,
    is_eligible_member: impl FnOnce(&Id) -> bool,
) -> Result<Id, AssignmentError> {
    match priority {
        Priority::P1 => match requested_id {
            Some(id) if id != manager_id => Err(AssignmentError::P1RequiresReviewingManager),
            _ => Ok(manager_id),
        },
        Priority::P2 | Priority::P3 | Priority::P4 => {
            let id = requested_id.ok_or(AssignmentError::AssigneeRequired)?;
            if is_eligible_member(&id) {
                Ok(id)
            } else {
                Err(AssignmentError::IneligibleMember)
            }
        }
    }
}

#[cfg(test)]
mod assignment_tests {
    use super::*;

    #[test]
    fn p1_goes_to_the_reviewing_manager_without_membership() {
        for requested in [None, Some(10)] {
            assert_eq!(
                select_assignee(Priority::P1, 10, requested, |_| false),
                Ok(10)
            );
        }
        assert_eq!(
            select_assignee(Priority::P1, 10, Some(20), |_| true),
            Err(AssignmentError::P1RequiresReviewingManager)
        );
    }

    #[test]
    fn other_priorities_require_an_eligible_member() {
        for priority in [Priority::P2, Priority::P3, Priority::P4] {
            let eligible = |id: &u32| *id == 20;
            assert_eq!(select_assignee(priority, 10, Some(20), eligible), Ok(20));
            assert_eq!(
                select_assignee(priority, 10, None, eligible),
                Err(AssignmentError::AssigneeRequired)
            );
            assert_eq!(
                select_assignee(priority, 10, Some(30), eligible),
                Err(AssignmentError::IneligibleMember)
            );
        }
    }
}

pub struct ReviewState {
    pub priority: Priority,
    pub version: i64,
    pub status: TicketStatus,
    pub has_assignee: bool,
    pub has_reviewer: bool,
    pub has_review_timestamp: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReviewError {
    VersionConflict,
    NotPendingReview,
    Assignment(AssignmentError),
}

/// Selects the assignee for a verified active manager's review.
pub fn review_ticket<Id: Eq>(
    ticket: &ReviewState,
    expected_version: i64,
    manager_id: Id,
    requested_id: Option<Id>,
    is_eligible_member: impl FnOnce(&Id) -> bool,
) -> Result<Id, ReviewError> {
    check_version(ticket.version, expected_version).map_err(|_| ReviewError::VersionConflict)?;
    if !is_pending_review(
        ticket.status,
        ticket.has_assignee,
        ticket.has_reviewer,
        ticket.has_review_timestamp,
    ) {
        return Err(ReviewError::NotPendingReview);
    }

    select_assignee(ticket.priority, manager_id, requested_id, is_eligible_member)
        .map_err(ReviewError::Assignment)
}

#[cfg(test)]
mod review_tests {
    use super::*;

    fn pending(priority: Priority) -> ReviewState {
        ReviewState {
            priority,
            version: 3,
            status: TicketStatus::Open,
            has_assignee: false,
            has_reviewer: false,
            has_review_timestamp: false,
        }
    }

    #[test]
    fn review_uses_stored_priority() {
        let critical = pending(Priority::P1);
        assert_eq!(review_ticket(&critical, 3, 10, None, |_| false), Ok(10));
        let high = pending(Priority::P2);
        assert_eq!(review_ticket(&high, 3, 10, Some(20), |id| *id == 20), Ok(20));
        assert_eq!(
            review_ticket(&high, 3, 10, Some(20), |_| false),
            Err(ReviewError::Assignment(AssignmentError::IneligibleMember))
        );
    }

    #[test]
    fn rejects_stale_and_already_reviewed_tickets() {
        let mut ticket = pending(Priority::P1);
        assert_eq!(
            review_ticket(&ticket, 2, 10, None, |_| false),
            Err(ReviewError::VersionConflict)
        );
        ticket.has_assignee = true;
        ticket.has_reviewer = true;
        ticket.has_review_timestamp = true;
        assert_eq!(
            review_ticket(&ticket, 3, 10, None, |_| false),
            Err(ReviewError::NotPendingReview)
        );
    }
}
