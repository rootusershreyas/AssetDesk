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

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
