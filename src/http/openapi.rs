use utoipa::OpenApi;
use crate::http::controllers;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::policy::create,
    controllers::policy::get,
    controllers::policy::delete,
    controllers::identity::create,
    controllers::identity::get,
    controllers::identity::delete,
    controllers::attachment::create,
    controllers::attachment::get,
    controllers::attachment::delete,
    controllers::token::token,
))]
pub struct ApiDoc;

