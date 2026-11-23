use super::model::Priority;
use super::validation::{ValidationError, validate_ticket_text};
use crate::features::categories::model::TicketType;

pub struct CreateTicketInput<Id> {
    pub ticket_type: TicketType,
    pub category_id: Id,
    pub priority: Priority,
    pub subject: String,
    pub description: String,
    pub custom_concern: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateInputError {
    InvalidText(ValidationError),
    MissingCustomConcern,
}

impl<Id> CreateTicketInput<Id> {
    /// Supply the stored name of the category looked up by category_id.
    pub fn validate(&self, category_name: &str) -> Result<(), CreateInputError> {
        let incident = self.ticket_type == TicketType::Incident;
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
            ticket_type: TicketType::Incident,
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
        input.ticket_type = TicketType::ServiceRequest;
        assert_eq!(input.validate("Others"), Ok(()));
        assert!(serde_json::from_str::<TicketType>(r#""unknown""#).is_err());
    }
}
