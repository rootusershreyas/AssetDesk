use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum OperationalStatus {
    #[default]
    InService,
    Maintenance,
    Retired,
}

impl OperationalStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::InService => "in_service",
            Self::Maintenance => "maintenance",
            Self::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Asset {
    pub id: Uuid,
    pub asset_tag: String,
    pub name: String,
    pub category: String,
    pub operational_status: OperationalStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// True only when in service and without an outstanding assignment.
    pub available: bool,
}
