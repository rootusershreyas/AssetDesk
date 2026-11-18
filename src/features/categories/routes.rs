use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::{dto::CreateCategory, repository};

fn error_response(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (
        status,
        Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        })),
    )
        .into_response()
}

pub async fn create_category(
    State(pool): State<PgPool>,
    payload: Result<Json<CreateCategory>, JsonRejection>,
) -> Response {
    let Json(input) = match payload {
        Ok(input) => input,
        Err(error) => {
            return error_response(
                error.status(),
                "invalid_request",
                "Provide a valid JSON category request.",
            );
        }
    };

    let input = match input.validate() {
        Ok(input) => input,
        Err(message) => {
            return error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                message,
            );
        }
    };

    match repository::insert_category(&pool, &input).await {
        Ok(category) => {
            let location = format!("/api/v1/categories/{}", category.id);

            (
                StatusCode::CREATED,
                [(header::LOCATION, location)],
                Json(category),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(error))
            if error.is_unique_violation()
                && error.constraint() == Some("categories_ticket_type_name_idx") =>
        {
            error_response(
                StatusCode::CONFLICT,
                "category_exists",
                "A category with this name already exists for this ticket type.",
            )
        }
        Err(_) => {
            tracing::error!("Category insertion failed");

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Could not create the category.",
            )
        }
    }
}
