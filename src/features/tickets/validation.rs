#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    InvalidSubject,
    InvalidDescription,
    InvalidExplanation,
}

pub fn validate_ticket_text(
    subject: &str,
    description: &str,
) -> Result<(), ValidationError> {
    let subject_length = subject.trim().chars().count();
    let description_length = description.trim().chars().count();

    if !(1..=160).contains(&subject_length) {
        return Err(ValidationError::InvalidSubject);
    }

    if !(1..=8_000).contains(&description_length) {
        return Err(ValidationError::InvalidDescription);
    }

    Ok(())
}

pub fn validate_reopening_reason(explanation: &str) -> Result<(), ValidationError> {
    if !(1..=4_000).contains(&explanation.trim().chars().count()) {
        return Err(ValidationError::InvalidExplanation);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_text_and_upper_bounds() {
        assert_eq!(validate_ticket_text("VPN issue", "Cannot connect"), Ok(()));
        assert_eq!(
            validate_ticket_text(&"é".repeat(160), &"x".repeat(8_000)),
            Ok(())
        );
    }

    #[test]
    fn rejects_blank_text() {
        assert_eq!(
            validate_ticket_text(" \n ", "Details"),
            Err(ValidationError::InvalidSubject)
        );
        assert_eq!(
            validate_ticket_text("Subject", " \t "),
            Err(ValidationError::InvalidDescription)
        );
    }

    #[test]
    fn rejects_text_over_upper_bounds() {
        assert_eq!(
            validate_ticket_text(&"x".repeat(161), "Details"),
            Err(ValidationError::InvalidSubject)
        );
        assert_eq!(
            validate_ticket_text("Subject", &"x".repeat(8_001)),
            Err(ValidationError::InvalidDescription)
        );
    }

    #[test]
    fn validates_reopening_reason() {
        assert_eq!(validate_reopening_reason("Issue persists"), Ok(()));
        assert_eq!(validate_reopening_reason(&"é".repeat(4_000)), Ok(()));

        for invalid in ["", " \n ", &"x".repeat(4_001)] {
            assert_eq!(
                validate_reopening_reason(invalid),
                Err(ValidationError::InvalidExplanation)
            );
        }
    }
}
