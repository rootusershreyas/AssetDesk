use sqlx::PgPool;

use super::{dto::CreateCategory, model::Category};

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
