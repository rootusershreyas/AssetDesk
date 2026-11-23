use crate::AppState;
use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::error::DatabaseError;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub account_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUser {
    name: String,
    email: String,
    role: String,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
struct UserPage {
    items: Vec<User>,
    limit: i64,
    offset: i64,
}

#[derive(Serialize)]
struct ApiError {
    error: &'static str,
    message: String,
}

struct UserApiError(StatusCode, &'static str, String);

impl IntoResponse for UserApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = (self.0, self.1, self.2);
        (
            status,
            Json(ApiError {
                error: code,
                message,
            }),
        )
            .into_response()
    }
}

impl UserApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, "bad_request", message.into())
    }

    fn validation(message: impl Into<String>) -> Self {
        Self(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            message.into(),
        )
    }

    fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "not_found", "User not found".into())
    }

    fn internal() -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "The request could not be completed".into(),
        )
    }
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user))
}

async fn create_user(
    State(state): State<AppState>,
    payload: Result<Json<CreateUser>, JsonRejection>,
) -> Result<impl IntoResponse, UserApiError> {
    let Json(payload) =
        payload.map_err(|_| UserApiError::bad_request("Request body must be valid JSON"))?;
    let name = payload.name.trim();
    let email = payload.email.trim().to_lowercase();
    let role = payload.role.trim();

    if name.is_empty() || name.chars().count() > 200 {
        return Err(UserApiError::validation(
            "name must contain 1 to 200 characters",
        ));
    }
    if email.chars().count() > 320 || !is_valid_email(&email) {
        return Err(UserApiError::validation(
            "email must be a valid email address",
        ));
    }
    if !matches!(role, "employee" | "it_manager" | "it_member") {
        return Err(UserApiError::validation(
            "role must be employee, it_manager, or it_member",
        ));
    }

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, role) VALUES ($1, $2, $3) \
         RETURNING id, name, email, role, account_status, created_at, updated_at",
    )
    .bind(name)
    .bind(email)
    .bind(role)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if error
            .as_database_error()
            .and_then(DatabaseError::code)
            .as_deref()
            == Some("23505")
        {
            UserApiError(
                StatusCode::CONFLICT,
                "email_exists",
                "A user with this email already exists".into(),
            )
        } else {
            tracing::error!(error = %error, "failed to create user");
            UserApiError::internal()
        }
    })?;

    let location = format!("/api/v1/users/{}", user.id);
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(user),
    ))
}

fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty()
        || local.len() > 64
        || domain.is_empty()
        || email.chars().any(char::is_whitespace)
        || domain.starts_with('.')
        || domain.ends_with('.')
        || !domain.contains('.')
        || email.matches('@').count() != 1
    {
        return false;
    }

    local
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(ch))
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, UserApiError> {
    let id = Uuid::parse_str(&id).map_err(|_| UserApiError::bad_request("id must be a UUID"))?;
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, account_status, created_at, updated_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "failed to fetch user");
        UserApiError::internal()
    })?
    .ok_or_else(UserApiError::not_found)?;

    Ok(Json(user))
}

async fn list_users(
    State(state): State<AppState>,
    query: Result<Query<ListUsersQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<UserPage>, UserApiError> {
    let Query(query) = query.map_err(|_| UserApiError::bad_request("Invalid pagination query"))?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=100).contains(&limit) || offset < 0 {
        return Err(UserApiError::validation(
            "limit must be 1 to 100 and offset must be nonnegative",
        ));
    }

    let items = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, account_status, created_at, updated_at \
         FROM users ORDER BY created_at DESC, id ASC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "failed to list users");
        UserApiError::internal()
    })?;

    Ok(Json(UserPage {
        items,
        limit,
        offset,
    }))
}
