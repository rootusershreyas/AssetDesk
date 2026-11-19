use super::{
    dto::{CreateCategory, UpdateCategory},
    model::Category,
    repository,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum CategoryServiceError {
    Validation(&'static str),
    Conflict,
    Database(sqlx::Error),
}

pub async fn create_category(
    pool: &PgPool,
    input: CreateCategory,
) -> Result<Category, CategoryServiceError> {
    let input = input.validate().map_err(CategoryServiceError::Validation)?;

    match repository::insert_category(pool, &input).await {
        Ok(category) => Ok(category),

        Err(sqlx::Error::Database(error))
            if error.is_unique_violation() && error.constraint() == Some("categories_name_idx") =>
        {
            Err(CategoryServiceError::Conflict)
        }

        Err(error) => Err(CategoryServiceError::Database(error)),
    }
}
pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    input: UpdateCategory,
) -> Result<Option<Category>, CategoryServiceError> {
    let input = input.validate().map_err(CategoryServiceError::Validation)?;

    match repository::update_category(pool, id, &input).await {
        Ok(category) => Ok(category),

        Err(sqlx::Error::Database(error))
            if error.is_unique_violation() && error.constraint() == Some("categories_name_idx") =>
        {
            Err(CategoryServiceError::Conflict)
        }

        Err(sqlx::Error::Database(error))
            if error.is_check_violation()
                && error.constraint() == Some("categories_incident_excludes_hardware") =>
        {
            Err(CategoryServiceError::Validation(
                "Hardware categories belong under Service Request.",
            ))
        }

        Err(error) => Err(CategoryServiceError::Database(error)),
    }
}
