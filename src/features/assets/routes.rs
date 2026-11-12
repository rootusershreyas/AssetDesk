use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, header::LOCATION},
    routing::{get, post},
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    state::AppState,
};

use super::{
    dto::{AssetPage, AssetSort, CreateAsset, ListAssetsQuery, PatchAsset, SortOrder},
    model::{Asset, OperationalStatus},
    repository,
};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/assets", post(create_asset).get(list_assets))
        .route("/assets/{id}", get(get_asset).patch(update_asset))
}

#[derive(OpenApi)]
#[openapi(
    paths(create_asset, list_assets, get_asset, update_asset),
    components(schemas(Asset, OperationalStatus, CreateAsset, PatchAsset, AssetPage, AssetSort, SortOrder, ErrorResponse)),
    tags((name = "assets", description = "Company equipment registration and availability"))
)]
pub(super) struct AssetApi;

#[utoipa::path(
    post, path = "/api/v1/assets", tag = "assets", request_body = CreateAsset,
    responses(
        (status = 201, description = "Asset registered", body = Asset, headers(("Location" = String, description = "Created asset URL"))),
        (status = 400, description = "Malformed JSON", body = ErrorResponse),
        (status = 409, description = "Duplicate asset tag", body = ErrorResponse),
        (status = 413, description = "Request body exceeds the size limit", body = ErrorResponse),
        (status = 415, description = "Content-Type must be application/json", body = ErrorResponse),
        (status = 422, description = "Invalid asset data", body = ErrorResponse),
        (status = 500, description = "Unexpected failure", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse)
    )
)]
async fn create_asset(
    State(state): State<AppState>,
    payload: Result<Json<CreateAsset>, JsonRejection>,
) -> Result<
    (
        StatusCode,
        [(axum::http::HeaderName, String); 1],
        Json<Asset>,
    ),
    AppError,
> {
    let Json(input) = payload.map_err(json_error)?;
    let asset = repository::create(&state.pool, input.validate()?).await?;
    let location = format!("/api/v1/assets/{}", asset.id);
    Ok((StatusCode::CREATED, [(LOCATION, location)], Json(asset)))
}

#[utoipa::path(
    get, path = "/api/v1/assets", tag = "assets", params(ListAssetsQuery),
    responses(
        (status = 200, description = "Matching assets with bounded pagination", body = AssetPage),
        (status = 400, description = "Invalid query syntax, filters or sort", body = ErrorResponse),
        (status = 422, description = "Invalid pagination or search values", body = ErrorResponse),
        (status = 500, description = "Unexpected failure", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse)
    )
)]
async fn list_assets(
    State(state): State<AppState>,
    query: Result<Query<ListAssetsQuery>, QueryRejection>,
) -> Result<Json<AssetPage>, AppError> {
    let Query(query) = query.map_err(|error| AppError::BadRequest(error.body_text()))?;
    let query = query.validate()?;
    let items = repository::list(&state.pool, &query).await?;
    Ok(Json(AssetPage {
        items,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[utoipa::path(
    get, path = "/api/v1/assets/{id}", tag = "assets", params(("id" = Uuid, Path, description = "Asset identifier")),
    responses(
        (status = 200, description = "Asset and current availability", body = Asset),
        (status = 400, description = "Malformed identifier", body = ErrorResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 500, description = "Unexpected failure", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse)
    )
)]
async fn get_asset(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<Asset>, AppError> {
    let Path(id) = path.map_err(|error| AppError::BadRequest(error.body_text()))?;
    Ok(Json(repository::get(&state.pool, id).await?))
}

#[utoipa::path(
    patch, path = "/api/v1/assets/{id}", tag = "assets", request_body = PatchAsset,
    params(("id" = Uuid, Path, description = "Asset identifier")),
    responses(
        (status = 200, description = "Asset updated", body = Asset),
        (status = 400, description = "Malformed identifier or JSON", body = ErrorResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 409, description = "Assigned equipment cannot be maintained or retired", body = ErrorResponse),
        (status = 413, description = "Request body exceeds the size limit", body = ErrorResponse),
        (status = 415, description = "Content-Type must be application/json", body = ErrorResponse),
        (status = 422, description = "Invalid, null, unknown or immutable field", body = ErrorResponse),
        (status = 500, description = "Unexpected failure", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse)
    )
)]
async fn update_asset(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    payload: Result<Json<PatchAsset>, JsonRejection>,
) -> Result<Json<Asset>, AppError> {
    let Path(id) = path.map_err(|error| AppError::BadRequest(error.body_text()))?;
    let Json(input) = payload.map_err(json_error)?;
    Ok(Json(
        repository::update(&state.pool, id, input.validate()?).await?,
    ))
}

fn json_error(error: JsonRejection) -> AppError {
    match error.status() {
        StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType,
        StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
        StatusCode::UNPROCESSABLE_ENTITY => AppError::Validation(error.body_text()),
        _ => AppError::BadRequest(error.body_text()),
    }
}
