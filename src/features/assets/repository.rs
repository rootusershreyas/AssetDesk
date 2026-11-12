use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;

use super::{
    dto::{Availability, CreateAsset, ListAssetsQuery, PatchAsset},
    model::{Asset, OperationalStatus},
};

const OPEN_ASSIGNMENT: &str =
    "EXISTS (SELECT 1 FROM assignments s WHERE s.asset_id = a.id AND s.returned_at IS NULL)";

const SELECT_ASSET: &str = "SELECT a.id, a.asset_tag, a.name, a.category, \
    a.operational_status, a.created_at, a.updated_at, \
    (a.operational_status = 'in_service' AND NOT EXISTS \
      (SELECT 1 FROM assignments s WHERE s.asset_id = a.id AND s.returned_at IS NULL)) \
    AS available FROM assets a";

pub(super) async fn create(pool: &PgPool, input: CreateAsset) -> Result<Asset, AppError> {
    let asset = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (id, asset_tag, name, category, operational_status) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, asset_tag, name, category, operational_status, created_at, updated_at, \
         (operational_status = 'in_service') AS available",
    )
    .bind(Uuid::new_v4())
    .bind(input.asset_tag)
    .bind(input.name)
    .bind(input.category)
    .bind(input.operational_status.as_str())
    .fetch_one(pool)
    .await
    .map_err(map_database_error)?;
    tracing::info!(asset_id = %asset.id, "Asset registered");
    Ok(asset)
}

pub(super) async fn get(pool: &PgPool, id: Uuid) -> Result<Asset, AppError> {
    sqlx::query_as::<_, Asset>(&format!("{SELECT_ASSET} WHERE a.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Asset"))
}

pub(super) async fn list(pool: &PgPool, query: &ListAssetsQuery) -> Result<Vec<Asset>, AppError> {
    let mut builder = QueryBuilder::<Postgres>::new(SELECT_ASSET);
    builder.push(" WHERE TRUE");
    if let Some(category) = &query.category {
        builder.push(" AND a.category = ").push_bind(category);
    }
    if let Some(status) = query.operational_status {
        builder
            .push(" AND a.operational_status = ")
            .push_bind(status.as_str());
    }
    if let Some(availability) = query.availability {
        match availability {
            Availability::Available => {
                builder
                    .push(" AND a.operational_status = 'in_service' AND NOT ")
                    .push(OPEN_ASSIGNMENT);
            }
            Availability::Assigned => {
                builder.push(" AND ").push(OPEN_ASSIGNMENT);
            }
            Availability::Unavailable => {
                builder.push(" AND a.operational_status IN ('maintenance', 'retired')");
            }
        }
    }
    if let Some(q) = &query.q {
        let pattern = literal_substring(q);
        builder
            .push(" AND (a.name ILIKE ")
            .push_bind(pattern.clone())
            .push(" ESCAPE '!' OR a.asset_tag ILIKE ")
            .push_bind(pattern)
            .push(" ESCAPE '!')");
    }
    // Only static strings selected by enums are used as SQL identifiers/directions.
    builder
        .push(" ORDER BY ")
        .push(query.sort.column())
        .push(query.order.sql())
        .push(", a.id")
        .push(query.order.sql())
        .push(" LIMIT ")
        .push_bind(query.limit)
        .push(" OFFSET ")
        .push_bind(query.offset);
    Ok(builder.build_query_as::<Asset>().fetch_all(pool).await?)
}

pub(super) async fn update(pool: &PgPool, id: Uuid, input: PatchAsset) -> Result<Asset, AppError> {
    let mut tx = pool.begin().await?;
    // C's issue/return operations must take this same asset lock before custody writes.
    let current =
        sqlx::query_as::<_, Asset>(&format!("{SELECT_ASSET} WHERE a.id = $1 FOR UPDATE OF a"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound("Asset"))?;

    let status = input
        .operational_status
        .unwrap_or(current.operational_status);
    if status != OperationalStatus::InService {
        let outstanding: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM assignments \
             WHERE asset_id = $1 AND returned_at IS NULL)",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        if outstanding {
            return Err(AppError::Conflict {
                code: "asset_already_assigned",
                message: "Return this asset before placing it into maintenance or retiring it.",
            });
        }
    }

    sqlx::query(
        "UPDATE assets SET name = $2, category = $3, operational_status = $4, \
         updated_at = clock_timestamp() WHERE id = $1",
    )
    .bind(id)
    .bind(input.name.unwrap_or(current.name))
    .bind(input.category.unwrap_or(current.category))
    .bind(status.as_str())
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, Asset>(&format!("{SELECT_ASSET} WHERE a.id = $1"))
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    tracing::info!(asset_id = %id, "Asset updated");
    Ok(updated)
}

fn map_database_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database) = &error
        && database.is_unique_violation()
        && database.constraint() == Some("assets_asset_tag_key")
    {
        return AppError::Conflict {
            code: "duplicate_asset_tag",
            message: "An asset with this tag already exists.",
        };
    }
    AppError::Database(error)
}

fn literal_substring(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for character in value.chars() {
        if matches!(character, '!' | '%' | '_') {
            pattern.push('!');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

#[cfg(test)]
mod tests {
    use super::literal_substring;

    #[test]
    fn search_metacharacters_are_literal_substrings() {
        assert_eq!(literal_substring("100%_!\\"), "%100!%!_!!\\%");
    }
}
