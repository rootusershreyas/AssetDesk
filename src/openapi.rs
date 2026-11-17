use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "AssetDesk API",
        version = "0.1.0",
        description = "Company IT/electronics inventory: TVs, remotes, laptops, HDMI connectors, Wi-Fi devices, chargers, UPS units and extensible future categories. B provides registration, queries and operational status. Location/deployment tracking is planned; A/C own access and custody integration."
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
