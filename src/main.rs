mod http;
mod models;
mod services;

use crate::http::controllers::{association, identity, principal, schema, token::token};
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository,
};
use crate::services::configuration::base::initialization_configuration_manager::InitializationConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::token_service::TokenService;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::openapi::ApiDoc;
use crate::services::backends::base::load_backend;
use crate::services::configuration::models::AppSettings;
use crate::services::identity_validator_provider::ExternalIdentityWatcher;
use crate::services::principal_service::PrincipalService;
use anyhow::Result;
use boxer_core::services::base::types::SchemaRepository;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8888);

    let cm = AppSettings::new()?;
    let current_backend = load_backend(cm.get_backend_type(), &cm).await?;

    let validator_provider = Arc::new(identity_validator_provider::new(current_backend.clone()));

    let configuration_watcher = validator_provider.clone();
    let _ = tokio::spawn(configuration_watcher.watch_for_identity_providers());
    info!("Configuration manager started");

    let schemas_repository: Arc<SchemaRepository> = current_backend.get_schemas_repository();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get_entities_repository();
    let principal_association_repository: Arc<PrincipalAssociationRepository> =
        current_backend.get_principal_association_repository();
    let identity_repository: Arc<IdentityRepository> = current_backend.get_identity_repository();

    let principal_service = Arc::new(PrincipalService::new(
        identity_repository.clone(),
        entities_repository.clone(),
        principal_association_repository.clone(),
        schemas_repository.clone(),
    ));

    let token_provider = Arc::new(TokenService::new(
        validator_provider.clone(),
        principal_service.clone(),
        cm.get_signing_key(),
    ));

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(token_provider.clone()))
            .app_data(Data::new(principal_service.clone()))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            .app_data(Data::new(principal_association_repository.clone()))
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
    .map_err(anyhow::Error::from)
}
