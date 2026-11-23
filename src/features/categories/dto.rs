use super::model::TicketType;
use serde::{Deserialize, Deserializer, Serialize};

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

        if let Some(description) = &self.description
            && description.chars().count() > 1000
        {
            return Err("Description must contain at most 1000 characters.");
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

#[derive(Debug, Default, PartialEq, Eq)]
pub enum PatchField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for PatchField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T> PatchField<T> {
    pub fn is_present(&self) -> bool {
        !matches!(self, Self::Missing)
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCategory {
    pub name: Option<String>,

    #[serde(default)]
    pub description: PatchField<String>,

    pub is_active: Option<bool>,
}

impl UpdateCategory {
    pub fn validate(mut self) -> Result<Self, &'static str> {
        if self.name.is_none() && !self.description.is_present() && self.is_active.is_none() {
            return Err("Provide at least one category field to update.");
        }

        if let Some(name) = &mut self.name {
            *name = name.trim().to_owned();

            if !(1..=80).contains(&name.chars().count()) {
                return Err("Category name must contain 1 to 80 characters.");
            }
        }

        if let PatchField::Value(description) = &self.description
            && description.chars().count() > 1000
        {
            return Err("Description must contain at most 1000 characters.");
        }

        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListCategoriesQuery {
    pub ticket_type: Option<TicketType>,

    #[serde(default = "default_limit")]
    pub limit: i64,

    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

impl ListCategoriesQuery {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1..=100).contains(&self.limit) {
            return Err("Limit must be between 1 and 100.");
        }

        if self.offset < 0 {
            return Err("Offset must be zero or greater.");
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: i64,
    pub offset: i64,
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
    #[test]
    fn accepts_valid_list_pagination() {
        let query = ListCategoriesQuery {
            ticket_type: Some(TicketType::Incident),
            limit: 20,
            offset: 0,
        };

        assert!(query.validate().is_ok());
    }

    #[test]
    fn rejects_zero_limit() {
        let query = ListCategoriesQuery {
            ticket_type: None,
            limit: 0,
            offset: 0,
        };

        assert_eq!(
            query.validate().unwrap_err(),
            "Limit must be between 1 and 100."
        );
    }

    #[test]
    fn rejects_limit_above_one_hundred() {
        let query = ListCategoriesQuery {
            ticket_type: None,
            limit: 101,
            offset: 0,
        };

        assert!(query.validate().is_err());
    }

    #[test]
    fn rejects_negative_offset() {
        let query = ListCategoriesQuery {
            ticket_type: None,
            limit: 20,
            offset: -1,
        };

        assert_eq!(
            query.validate().unwrap_err(),
            "Offset must be zero or greater."
        );
    }

    #[test]
    fn rejects_empty_category_update() {
        let input: UpdateCategory = serde_json::from_value(serde_json::json!({})).unwrap();

        assert_eq!(
            input.validate().unwrap_err(),
            "Provide at least one category field to update."
        );
    }

    #[test]
    fn trims_updated_category_name() {
        let input: UpdateCategory =
            serde_json::from_value(serde_json::json!({ "name": "  Accounts  " })).unwrap();

        let input = input.validate().unwrap();
        assert_eq!(input.name.as_deref(), Some("Accounts"));
    }

    #[test]
    fn distinguishes_missing_null_and_present_description() {
        let missing: UpdateCategory = serde_json::from_value(serde_json::json!({
            "is_active": false
        }))
        .unwrap();
        let null: UpdateCategory =
            serde_json::from_value(serde_json::json!({ "description": null })).unwrap();
        let value: UpdateCategory =
            serde_json::from_value(serde_json::json!({ "description": "Updated" })).unwrap();

        assert_eq!(missing.description, PatchField::Missing);
        assert_eq!(null.description, PatchField::Null);
        assert_eq!(value.description, PatchField::Value("Updated".to_owned()));
    }
}
