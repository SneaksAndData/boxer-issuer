use crate::http::controllers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::policy::post,
    controllers::policy::get,
    controllers::policy::delete,
    controllers::identity::post,
    controllers::identity::get,
    controllers::identity::delete,
    controllers::attachment::post,
    controllers::attachment::get,
    controllers::attachment::delete,
    controllers::token::token,
))]
pub struct ApiDoc;
