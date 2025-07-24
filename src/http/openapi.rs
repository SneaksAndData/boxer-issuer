use crate::http::controllers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::identity::post,
    controllers::identity::get,
    controllers::identity::delete,
    controllers::token::token,
    controllers::schema::post,
    controllers::schema::get,
    controllers::schema::delete,
    controllers::principal::post,
    controllers::principal::get,
    controllers::principal::delete,
    controllers::association::post,
    controllers::association::get,
))]
pub struct ApiDoc;
