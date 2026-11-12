mod dto;
mod model;
mod repository;
mod routes;

use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

pub use dto::{AssetPage, CreateAsset, ListAssetsQuery, PatchAsset};
pub use model::{Asset, OperationalStatus};

pub fn router() -> Router<AppState> {
    routes::router()
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    routes::AssetApi::openapi()
}
