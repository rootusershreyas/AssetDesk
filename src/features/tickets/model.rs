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