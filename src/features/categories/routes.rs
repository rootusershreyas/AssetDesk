use super::{
    dto::{CreateCategory, ListCategoriesQuery, Page, UpdateCategory},
    repository,
};
use axum::extract::{Path, rejection::PathRejection};
use axum::extract::{Query, rejection::QueryRejection};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

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

pub async fn list_categories(
    State(pool): State<PgPool>,
    query: Result<Query<ListCategoriesQuery>, QueryRejection>,
) -> Response {
    let Query(query) = match query {
        Ok(query) => query,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_query",
                "Provide valid ticket_type, limit and offset parameters.",
            );
        }
    };

    if let Err(message) = query.validate() {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            message,
        );
    }

    match repository::list_categories(&pool, &query).await {
        Ok(items) => Json(Page {
            items,
            limit: query.limit,
            offset: query.offset,
        })
        .into_response(),

        Err(_) => {
            tracing::error!("Category listing failed");

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Could not retrieve categories.",
            )
        }
    }
}

pub async fn get_category(
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Response {
    let Path(id) = match path {
        Ok(path) => path,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_id",
                "Category ID must be a valid UUID.",
            );
        }
    };

    match repository::find_category(&pool, id).await {
        Ok(Some(category)) => Json(category).into_response(),
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "category_not_found",
            "Category was not found.",
        ),
        Err(_) => {
            tracing::error!("Category lookup failed");

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Could not retrieve the category.",
            )
        }
    }
}

pub async fn update_category(
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, PathRejection>,
    payload: Result<Json<UpdateCategory>, JsonRejection>,
) -> Response {
    let Path(id) = match path {
        Ok(path) => path,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_id",
                "Category ID must be a valid UUID.",
            );
        }
    };

    let Json(input) = match payload {
        Ok(input) => input,
        Err(error) => {
            return error_response(
                error.status(),
                "invalid_request",
                "Provide a valid JSON category update.",
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

    match repository::update_category(&pool, id, &input).await {
        Ok(Some(category)) => Json(category).into_response(),
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "category_not_found",
            "Category was not found.",
        ),
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
        Err(sqlx::Error::Database(error))
            if error.is_check_violation()
                && error.constraint() == Some("categories_incident_excludes_hardware") =>
        {
            error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                "Hardware categories belong under Service Request.",
            )
        }
        Err(_) => {
            tracing::error!(category_id = %id, "Category update failed");

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Could not update the category.",
            )
        }
    }
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
