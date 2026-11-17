use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::error::AppError;

use super::model::{Asset, OperationalStatus};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateAsset {
    #[schema(example = "TV-001", min_length = 1, max_length = 64)]
    pub asset_tag: String,
    #[schema(example = "Meeting room TV", min_length = 1, max_length = 120)]
    pub name: String,
    /// Extensible category: tv, tv_remote, laptop, hdmi_connector, wifi_device,
    /// laptop_charger, ups or another nonempty device/accessory category.
    #[schema(example = "tv", min_length = 1, max_length = 60)]
    pub category: String,
    #[serde(default)]
    pub operational_status: OperationalStatus,
}

impl CreateAsset {
    pub(super) fn validate(mut self) -> Result<Self, AppError> {
        self.asset_tag = self.asset_tag.trim().to_uppercase();
        validate_text(&self.asset_tag, "asset_tag", 64)?;
        self.name = normalized_text(&self.name, "name", 120)?;
        self.category = normalized_text(&self.category, "category", 60)?;
        Ok(self)
    }
}

/// Omitted fields retain their current values; explicit null is rejected.
/// Asset tags are immutable and consequently are not accepted here.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PatchAsset {
    #[serde(default, deserialize_with = "non_null")]
    #[schema(nullable = false, min_length = 1, max_length = 120)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    #[schema(nullable = false, min_length = 1, max_length = 60)]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    #[schema(nullable = false)]
    pub operational_status: Option<OperationalStatus>,
}

impl PatchAsset {
    pub(super) fn validate(mut self) -> Result<Self, AppError> {
        if self.name.is_none() && self.category.is_none() && self.operational_status.is_none() {
            return Err(AppError::Validation(
                "Provide at least one asset field to update.".into(),
            ));
        }
        self.name = self
            .name
            .map(|name| normalized_text(&name, "name", 120))
            .transpose()?;
        self.category = self
            .category
            .map(|category| normalized_text(&category, "category", 60))
            .transpose()?;
        Ok(self)
    }
}

fn non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)?
        .ok_or_else(|| serde::de::Error::custom("Asset fields cannot be null."))
        .map(Some)
}

fn normalized_text(value: &str, field: &str, max: usize) -> Result<String, AppError> {
    let value = value.trim();
    validate_text(value, field, max)?;
    Ok(value.to_owned())
}

fn validate_text(value: &str, field: &str, max: usize) -> Result<(), AppError> {
    if value.is_empty() || value.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{field} must contain between 1 and {max} characters after trimming."
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum Availability {
    #[default]
    Available,
    Assigned,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetSort {
    AssetTag,
    Name,
    #[default]
    CreatedAt,
}

impl AssetSort {
    pub(super) fn column(self) -> &'static str {
        match self {
            Self::AssetTag => "a.asset_tag",
            Self::Name => "a.name",
            Self::CreatedAt => "a.created_at",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl SortOrder {
    pub(super) fn sql(self) -> &'static str {
        match self {
            Self::Asc => " ASC",
            Self::Desc => " DESC",
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(default, deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct ListAssetsQuery {
    /// Literal case-insensitive substring in asset name or tag.
    pub q: Option<String>,
    pub category: Option<String>,
    pub operational_status: Option<OperationalStatus>,
    #[param(inline)]
    pub(super) availability: Option<Availability>,
    pub sort: AssetSort,
    pub order: SortOrder,
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub limit: i64,
    #[param(minimum = 0, default = 0)]
    pub offset: i64,
}

impl Default for ListAssetsQuery {
    fn default() -> Self {
        Self {
            q: None,
            category: None,
            operational_status: None,
            availability: None,
            sort: AssetSort::default(),
            order: SortOrder::default(),
            limit: 20,
            offset: 0,
        }
    }
}

impl ListAssetsQuery {
    pub(super) fn validate(mut self) -> Result<Self, AppError> {
        if !(1..=100).contains(&self.limit) {
            return Err(AppError::Validation(
                "limit must be between 1 and 100.".into(),
            ));
        }
        if self.offset < 0 {
            return Err(AppError::Validation("offset must be nonnegative.".into()));
        }
        self.category = self
            .category
            .map(|category| normalized_text(&category, "category", 60))
            .transpose()?;
        if self.q.as_ref().is_some_and(|q| q.chars().count() > 120) {
            return Err(AppError::Validation(
                "q must be at most 120 characters.".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetPage {
    pub items: Vec<Asset>,
    pub limit: i64,
    pub offset: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_tag_is_validated_after_uppercase_expansion() {
        let asset = CreateAsset {
            asset_tag: format!("{}ß", "a".repeat(63)),
            name: "Laptop".into(),
            category: "laptop".into(),
            operational_status: OperationalStatus::InService,
        };
        assert!(asset.validate().is_err());
    }

    #[test]
    fn unicode_limits_count_characters_not_utf8_bytes() {
        let asset = CreateAsset {
            asset_tag: " lap-1 ".into(),
            name: "é".repeat(120),
            category: " laptop ".into(),
            operational_status: OperationalStatus::InService,
        }
        .validate()
        .unwrap();
        assert_eq!(asset.asset_tag, "LAP-1");
        assert_eq!(asset.category, "laptop");
    }

    #[test]
    fn patch_distinguishes_omission_from_null_and_rejects_immutable_fields() {
        let omitted: PatchAsset = serde_json::from_value(json!({"name": "Laptop"})).unwrap();
        assert!(omitted.category.is_none());
        for value in [
            json!({"name": null}),
            json!({"category": null}),
            json!({"operational_status": null}),
            json!({"asset_tag": "NEW"}),
            json!({"available": true}),
        ] {
            assert!(serde_json::from_value::<PatchAsset>(value).is_err());
        }
        assert!(
            serde_json::from_value::<PatchAsset>(json!({}))
                .unwrap()
                .validate()
                .is_err()
        );
    }

    #[test]
    fn explicit_null_for_each_patch_type_is_a_semantic_json_error() {
        for input in [
            r#"{"name":null}"#,
            r#"{"category":null}"#,
            r#"{"operational_status":null}"#,
        ] {
            let error = serde_json::from_str::<PatchAsset>(input).unwrap_err();
            assert!(error.is_data(), "Unexpected JSON error category: {error}");
        }
    }

    #[test]
    fn list_rejects_unbounded_and_negative_pagination() {
        let excessive = ListAssetsQuery {
            limit: 101,
            ..Default::default()
        };
        assert!(excessive.validate().is_err());
        let negative = ListAssetsQuery {
            offset: -1,
            ..Default::default()
        };
        assert!(negative.validate().is_err());
    }
}
