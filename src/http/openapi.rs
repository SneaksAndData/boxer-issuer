use crate::http::controllers;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

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
    controllers::provider::delete_provider,
    controllers::token::token,
),
    modifiers(&SecurityAddon))]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme("external", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
    }
}
