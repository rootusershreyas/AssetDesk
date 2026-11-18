use super::model::TicketType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCategory {
    pub ticket_type: TicketType,
    pub name: String,
    pub description: Option<String>,
}

impl CreateCategory {
    pub fn validate(mut self) -> Result<Self, &'static str> {
        self.name = self.name.trim().to_owned();

        if !(1..=80).contains(&self.name.chars().count()) {
            return Err("Category name must contain 1 to 80 characters.");
        }

        if let Some(description) = &self.description {
            if description.chars().count() > 1000 {
                return Err("Description must contain at most 1000 characters.");
            }
        }

        let normalized_name = self
            .name
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        if self.ticket_type == TicketType::Incident
            && matches!(
                normalized_name.as_str(),
                "hardware" | "hardware issue" | "hardware request"
            )
        {
            return Err("Hardware categories belong under Service Request.");
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_hardware_categories_under_incident() {
        for name in [
            "Hardware",
            "  HARDWARE  ",
            "Hardware Issue",
            "Hardware   Issue",
            "Hardware Request",
        ] {
            let input = CreateCategory {
                ticket_type: TicketType::Incident,
                name: name.to_owned(),
                description: None,
            };

            assert_eq!(
                input.validate().unwrap_err(),
                "Hardware categories belong under Service Request."
            );
        }
    }

    #[test]
    fn accepts_hardware_request_under_service_request() {
        let input = CreateCategory {
            ticket_type: TicketType::ServiceRequest,
            name: "Hardware Request".to_owned(),
            description: None,
        };

        assert!(input.validate().is_ok());
    }

    #[test]
    fn accepts_others_under_incident() {
        let input = CreateCategory {
            ticket_type: TicketType::Incident,
            name: "Others".to_owned(),
            description: Some("IT incidents outside the listed categories.".to_owned()),
        };

        assert!(input.validate().is_ok());
    }

    #[test]
    fn trims_category_name() {
        let input = CreateCategory {
            ticket_type: TicketType::Incident,
            name: "  Network  ".to_owned(),
            description: None,
        };

        let category = input.validate().unwrap();
        assert_eq!(category.name, "Network");
        assert_eq!(category.ticket_type, TicketType::Incident);
    }

    #[test]
    fn rejects_blank_category_name() {
        let input = CreateCategory {
            ticket_type: TicketType::ServiceRequest,
            name: "   ".to_owned(),
            description: None,
        };

        assert!(input.validate().is_err());
    }
}
