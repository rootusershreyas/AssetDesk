use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AssetDesk API",
        version = "0.1.0",
        description = "B module: asset management and PostgreSQL readiness. Employee and assignment APIs are implemented separately by A and C."
    ),
    paths(crate::app::health, crate::app::ready),
    components(schemas(
        crate::app::HealthResponse,
        crate::error::ErrorResponse,
        crate::error::ErrorDetails
    ))
)]
struct OperationsApi;

pub fn document() -> utoipa::openapi::OpenApi {
    let mut document = OperationsApi::openapi();
    document.merge(crate::features::assets::openapi());
    document
}
