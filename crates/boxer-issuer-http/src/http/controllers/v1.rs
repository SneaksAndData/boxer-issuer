use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::http::middleware::audit::audit_scope::AuditScope;
use std::sync::Arc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub mod identity;
pub mod principal;
pub mod provider;

pub mod schema;
pub mod token;

#[derive(OpenApi)]
#[openapi(paths(
    identity::post_identity,
    identity::get_identity,
    identity::delete_identity,
    schema::post_schema,
    schema::get_schema,
    schema::delete_schema,
    principal::post_principal,
    principal::get_principal,
    provider::post_provider,
    provider::get_provider,
    provider::delete_provider,
    token::token,
),
    modifiers(&SecurityAddon))]
pub struct ApiV1;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme("external", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
        components.add_security_scheme("internal", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    }
}

pub fn urls(writer: Arc<dyn AuditWriter>) -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .service(identity::crud())
        .service(schema::crud())
        .service(principal::crud())
        .service(provider::crud())
        .service(web::scope("").service(token::token).with_initial_audit_scope(writer))
}
