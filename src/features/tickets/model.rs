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
    Resolved,
    Closed,
}
impl TicketStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::InProgress)
                | (Self::InProgress, Self::Resolved)
                | (Self::Resolved, Self::Closed)
                | (Self::Resolved, Self::Open)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TicketStatus;

    #[test]
    fn only_documented_transitions_are_allowed() {
        use TicketStatus::*;

        let states = [Open, InProgress, Resolved, Closed];
        let allowed = [
            (Open, InProgress),
            (InProgress, Resolved),
            (Resolved, Closed),
            (Resolved, Open),
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
        for status in [Open, InProgress, Resolved, Closed] {
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
