pub mod http;
pub mod models;
pub mod services;

use actix_web::dev::Server;
use actix_web::middleware::{from_fn, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::services::principal_service::PrincipalServiceTrait;
use crate::services::token_service::TokenProvider;
use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::http::middleware::logging::custom_error_logging;
use boxer_core::services::audit::AuditService;
use boxer_core::services::backends::kubernetes::kubernetes_repository::schema_repository::SchemaRepository;
use http::controllers::v1;
use http::health;
use http::openapi::ApiDoc;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use services::backends::base::IssuerBackend;
use services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use services::backends::kubernetes::identity_repository::IdentityRepository;
use services::backends::kubernetes::principal_repository::PrincipalRepository;
use services::configuration::models::AppSettings;

pub fn start_api_server(
    current_backend: Arc<dyn IssuerBackend>,
    token_provider: Arc<dyn TokenProvider>,
    audit_service: Arc<dyn AuditService>,
    audit_writer: Arc<dyn AuditWriter>,
    readiness_state: Arc<AtomicBool>,
    principal_service: Arc<dyn PrincipalServiceTrait>,
    cm: AppSettings,
) -> Result<Server, actix_web::Error> {
    let schemas_repository: Arc<SchemaRepository> = current_backend.get();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get();
    let identity_repository: Arc<IdentityRepository> = current_backend.get();
    let identity_provider_repository: Arc<IdentityProviderRepository> = current_backend.get();

    info!(host:? = &cm.listen_address.ip(); "listening on {}:{}", &cm.listen_address.ip(), &cm.listen_address.port());
    let server_builder = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(Logger::default())
            .wrap(from_fn(custom_error_logging))
            .app_data(Data::new(token_provider.clone()))
            .app_data(Data::new(principal_service.clone()))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            .app_data(Data::new(identity_provider_repository.clone()))
            .app_data(Data::new(audit_service.clone()))
            .app_data(Data::new(readiness_state.clone()))
            .service(v1::urls(audit_writer.clone()))
            .service(health::urls())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(cm.listen_address.clone())?;
    Ok(server_builder.run())
}
