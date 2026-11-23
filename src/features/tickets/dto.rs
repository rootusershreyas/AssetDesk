use super::model::Priority;
use super::validation::{ValidationError, validate_ticket_text};
use crate::features::categories::model::TicketType;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListTicketsQuery {
    pub status: Option<super::model::TicketStatus>,
    pub priority: Option<Priority>,
    pub ticket_type: Option<TicketType>,
    pub category_id: Option<uuid::Uuid>,
    #[serde(default = "default_ticket_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

pub(crate) fn default_ticket_limit() -> i64 {
    20
}

#[cfg(test)]
mod list_query_tests {
    use super::*;
    use axum::{extract::Query, http::Uri};

    #[test]
    fn accepts_typed_query_and_rejects_invalid_or_caller_selected_identity() {
        let parse =
            |url: &str| Query::<ListTicketsQuery>::try_from_uri(&url.parse::<Uri>().unwrap());
        let q = parse("/?status=awaiting_employee_confirmation&priority=p3&ticket_type=incident&category_id=00000000-0000-0000-0000-000000000001").unwrap().0;
        assert_eq!(
            q.status,
            Some(super::super::model::TicketStatus::AwaitingEmployeeConfirmation)
        );
        assert_eq!(q.priority, Some(Priority::P3));
        assert_eq!(q.ticket_type, Some(TicketType::Incident));
        assert_eq!(q.category_id, Some(uuid::Uuid::from_u128(1)));
        assert_eq!((q.limit, q.offset), (20, 0));
        for url in [
            "/?requester_id=1",
            "/?status=closed",
            "/?priority=p5",
            "/?category_id=bad",
            "/?limit=many",
        ] {
            assert!(parse(url).is_err(), "{url}");
        }
    }
}

#[cfg(test)]
mod creation_json_tests {
    use super::*;
    use serde_json::{Value, json};
    use uuid::Uuid;

    fn request() -> Value {
        json!({
            "ticket_type": "incident",
            "category_id": "00000000-0000-0000-0000-000000000001",
            "priority": "p2",
            "subject": "VPN",
            "description": "Connection drops"
        })
    }

    #[test]
    fn creation_json_preserves_employee_priority_and_validates_category_specific_concern() {
        let mut body = request();
        let parsed: CreateTicketInput<Uuid> = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(parsed.category_id, Uuid::from_u128(1));
        assert_eq!(parsed.priority, Priority::P2);
        assert_eq!(parsed.validate("VPN"), Ok(()));
        assert_eq!(
            parsed.validate("Others"),
            Err(CreateInputError::MissingCustomConcern)
        );
        body["custom_concern"] = json!("Disconnects after sleep");
        assert_eq!(
            serde_json::from_value::<CreateTicketInput<Uuid>>(body)
                .unwrap()
                .validate("Others"),
            Ok(())
        );
    }

    #[test]
    fn creation_json_rejects_invalid_fields_and_server_managed_values() {
        for field in [
            "requester_id",
            "assignee_id",
            "status",
            "ticket_number",
            "version",
            "reviewed_by",
        ] {
            let mut body = request();
            body[field] = json!("caller supplied");
            assert!(
                serde_json::from_value::<CreateTicketInput<Uuid>>(body).is_err(),
                "{field}"
            );
        }
        for (field, value) in [
            ("category_id", "bad-id"),
            ("priority", "p5"),
            ("ticket_type", "other"),
        ] {
            let mut body = request();
            body[field] = json!(value);
            assert!(
                serde_json::from_value::<CreateTicketInput<Uuid>>(body).is_err(),
                "{field}"
            );
        }
        let mut body = request();
        body.as_object_mut().unwrap().remove("priority");
        assert!(serde_json::from_value::<CreateTicketInput<Uuid>>(body).is_err());
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewTicketInput {
    pub expected_version: i64,
    pub assignee_id: Option<uuid::Uuid>,
}

#[cfg(test)]
mod review_json_tests {
    use super::ReviewTicketInput;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn review_accepts_only_version_and_optional_assignee() {
        let member = Uuid::from_u128(42);
        let parse = serde_json::from_value::<ReviewTicketInput>;
        let p1 = parse(json!({"expected_version": 1})).unwrap();
        assert_eq!((p1.expected_version, p1.assignee_id), (1, None));
        let assigned = parse(json!({"expected_version": 3, "assignee_id": member})).unwrap();
        assert_eq!((assigned.expected_version, assigned.assignee_id), (3, Some(member)));
        for body in [
            json!({}),
            json!({"expected_version": "1"}),
            json!({"expected_version": 1, "assignee_id": "bad-id"}),
            json!({"expected_version": 1, "manager_id": member}),
            json!({"expected_version": 1, "priority": "p4"}),
        ] {
            assert!(parse(body).is_err());
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum TicketStatusInput {
    Open { expected_version: i64, explanation: String },
    Resolved { expected_version: i64 },
    InProgress { expected_version: i64 },
    AwaitingEmployeeConfirmation { expected_version: i64, resolution: String, root_cause: Option<String> },
}

#[cfg(test)]
mod status_input_tests {
    use super::TicketStatusInput;
    use serde_json::json;

    #[test]
    fn starting_work_requires_a_version_and_rejects_caller_identity() {
        for body in [
            json!({"status": "in_progress"}),
            json!({"status": "in_progress", "expected_version": 1, "actor_id": "caller"}),
            json!({"status": "in_progress", "expected_version": 1, "resolution": "done"}),
        ] {
            assert!(serde_json::from_value::<TicketStatusInput>(body).is_err());
        }
    }
}
