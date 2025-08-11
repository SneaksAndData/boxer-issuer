pub mod http;
mod models;
mod services;

use crate::services::configuration::base::initialization_configuration_manager::InitializationConfigurationManager;
use crate::services::token_service::TokenService;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::controllers::token::token;
use crate::http::controllers::{identity, principal, provider, schema};
use crate::http::openapi::ApiDoc;
use crate::services::backends::base::load_backend;
use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use crate::services::configuration::models::AppSettings;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::principal_service::PrincipalService;
use anyhow::Result;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8888);

    let cm = AppSettings::new()?;
    let current_backend = load_backend(cm.get_backend_type(), &cm).await?;

    let validator_provider: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync> = current_backend.get();

    info!("Configuration manager started");

    let schemas_repository: Arc<SchemaRepository> = current_backend.get();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get();
    let identity_repository: Arc<IdentityRepository> = current_backend.get();
    let identity_provider_repository: Arc<IdentityProviderRepository> = current_backend.get();

    let principal_service = Arc::new(PrincipalService::new(
        identity_repository.clone(),
        entities_repository.clone(),
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
            .app_data(Data::new(identity_provider_repository.clone()))
            .service(token)
            .service(identity::crud())
            .service(schema::crud())
            .service(principal::crud())
            .service(provider::crud())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
