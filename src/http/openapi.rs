use crate::http::controllers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::identity::post_identity,
    controllers::identity::get_identity,
    controllers::identity::delete_identity,
    controllers::schema::post_schema,
    controllers::schema::get_schema,
    controllers::schema::delete_schema,
    controllers::principal::post_principal,
    controllers::principal::get_principal,
    controllers::provider::post_provider,
    controllers::provider::get_provider,
    controllers::token::token,
    controllers::association::post_association,
    controllers::association::get_association,
))]
pub struct ApiDoc;
