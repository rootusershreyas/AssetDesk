use super::{
    dto::{CreateCategory, ListCategoriesQuery, UpdateCategory},
    model::Category,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_category(
    pool: &PgPool,
    input: &CreateCategory,
) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (ticket_type, name, description)
        VALUES ($1, $2, $3)
        RETURNING
            id,
            ticket_type,
            name,
            description,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(&input.ticket_type)
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(pool)
    .await
}

pub async fn find_category(pool: &PgPool, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT
            id,
            ticket_type,
            name,
            description,
            is_active,
            created_at,
            updated_at
        FROM categories
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
pub async fn list_categories(
    pool: &PgPool,
    query: &ListCategoriesQuery,
) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT
            id,
            ticket_type,
            name,
            description,
            is_active,
            created_at,
            updated_at
        FROM categories
        WHERE ($1::text IS NULL OR ticket_type = $1)
        ORDER BY lower(name), id
        LIMIT $2
        OFFSET $3
        "#,
    )
    .bind(query.ticket_type.as_ref())
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(pool)
    .await
}

pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    input: &UpdateCategory,
) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET
            name = COALESCE($2, name),
            description = CASE WHEN $3 THEN $4 ELSE description END,
            is_active = COALESCE($5, is_active),
            updated_at = now()
        WHERE id = $1
        RETURNING
            id,
            ticket_type,
            name,
            description,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(&input.name)
    .bind(input.description.is_present())
    .bind(input.description.value())
    .bind(input.is_active)
    .fetch_optional(pool)
    .await
}
