use super::{dto::CreateCategory, model::Category};
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
