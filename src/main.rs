mod http;
mod models;
mod services;

use crate::http::controllers::{association, identity, principal, schema, token::token};
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository, SchemaRepository,
};
use crate::services::configuration_manager::ConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::token_service::TokenService;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::io::Result;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::openapi::ApiDoc;
use crate::services::backends::base::{load_backend, Backend};
use crate::services::principal_service::PrincipalService;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8888);
    let validator_provider = Arc::new(identity_validator_provider::new());
    let cm = Arc::clone(&validator_provider);
    let secret = Arc::new(cm.get_signing_key());
    let current_backend = load_backend(&cm);

    let _ = tokio::spawn(cm.watch_for_identity_providers());
    info!("Configuration manager started");

    // Replace hash maps with factory methods here
    let schemas_repository: Arc<SchemaRepository> = current_backend.get_schemas_repository();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get_entities_repository();
    let principal_association_repository: Arc<PrincipalAssociationRepository> =
        current_backend.get_principal_association_repository();
    let identity_repository: Arc<IdentityRepository> = current_backend.get_identity_repository();

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        let principal_service = Arc::new(PrincipalService::new(
            identity_repository.clone(),
            entities_repository.clone(),
            principal_association_repository.clone(),
            schemas_repository.clone(),
        ));
        let token_provider = Arc::new(TokenService::new(
            validator_provider.clone(),
            principal_service.clone(),
            Arc::clone(&secret),
        ));
        App::new()
            .app_data(Data::new(token_provider))
            .app_data(Data::new(principal_service))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            .app_data(Data::new(principal_association_repository.clone()))
            // Token endpoint
            .service(token)
            .service(identity::crud())
            .service(schema::crud())
            .service(principal::crud())
            .service(association::crud())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
}
