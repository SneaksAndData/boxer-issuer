use boxer_issuer::http::controllers::v1::ApiV1;
use boxer_issuer::http::health;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::get_health,
        health::get_health_probe
    ),
    nest(
        (path = "/api/v1", api = ApiV1)
    ),
)]
pub struct ApiDoc;
