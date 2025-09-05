use crate::http::controllers::v1::ApiV1;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/api/v1", api = ApiV1)
    ),
)]
pub struct ApiDoc;
