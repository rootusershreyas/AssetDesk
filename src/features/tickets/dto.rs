use super::model::Priority;
use super::validation::{ValidationError, validate_ticket_text};

pub struct CreateTicketInput<Id> {
    pub ticket_type: String,
    pub category_id: Id,
    pub priority: Priority,
    pub subject: String,
    pub description: String,
    pub custom_concern: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateInputError {
    InvalidTicketType,
    InvalidText(ValidationError),
    MissingCustomConcern,
}

impl<Id> CreateTicketInput<Id> {
    /// Supply the stored name of the category looked up by category_id.
    pub fn validate(&self, category_name: &str) -> Result<(), CreateInputError> {
        let incident = match self.ticket_type.as_str() {
            "incident" => true,
            "service_request" => false,
            _ => return Err(CreateInputError::InvalidTicketType),
        };
        validate_ticket_text(&self.subject, &self.description)
            .map_err(CreateInputError::InvalidText)?;
        if incident
            && category_name.trim().eq_ignore_ascii_case("Others")
            && self
                .custom_concern
                .as_deref()
                .is_none_or(|text| text.trim().is_empty())
        {
            return Err(CreateInputError::MissingCustomConcern);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn incident() -> CreateTicketInput<u32> {
        CreateTicketInput {
            ticket_type: "incident".into(),
            category_id: 1,
            priority: Priority::P2,
            subject: "Login".into(),
            description: "Login interrupted after pairing".into(),
            custom_concern: None,
        }
    }

    #[test]
    fn incident_others_requires_a_specific_concern() {
        let mut input = incident();
        assert_eq!(
            input.validate("Others"),
            Err(CreateInputError::MissingCustomConcern)
        );
        input.custom_concern = Some("   ".into());
        assert_eq!(
            input.validate("Others"),
            Err(CreateInputError::MissingCustomConcern)
        );
        input.custom_concern = Some("Pairing my headset locks the session".into());
        assert_eq!(input.validate("Others"), Ok(()));
    }

    #[test]
    fn ordinary_incidents_and_service_requests_can_omit_concern() {
        let mut input = incident();
        assert_eq!(input.validate("Login Issue"), Ok(()));
        input.ticket_type = "service_request".into();
        assert_eq!(input.validate("Others"), Ok(()));
        input.ticket_type = "unknown".into();
        assert_eq!(
            input.validate("Others"),
            Err(CreateInputError::InvalidTicketType)
        );
    }
}
