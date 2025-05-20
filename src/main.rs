mod http;
mod models;
mod services;

use crate::http::controllers::{association, attachment, identity, policy, principal, schema, token::token};
use crate::services::base::upsert_repository::{
    IdentityRepository, PolicyAttachmentRepository, PolicyRepository, PrincipalAssociationRepository,
    PrincipalsRepository, SchemaRepository,
};
use crate::services::configuration_manager::ConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::token_service::TokenService;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::collections::HashMap;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::openapi::ApiDoc;
use crate::services::principal_service::PrincipalService;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8888);
    let validator_provider = Arc::new(identity_validator_provider::new());
    let cm = Arc::clone(&validator_provider);
    let secret = Arc::new(cm.get_signing_key());

    let _ = tokio::spawn(cm.watch_for_identity_providers());
    info!("Configuration manager started");

    // Replace hash maps with factory methods here
    let policy_repository: Arc<PolicyRepository> = Arc::new(RwLock::new(HashMap::new()));
    let policy_attachments_repository: Arc<PolicyAttachmentRepository> = Arc::new(RwLock::new(HashMap::new()));
    let identity_repository: Arc<IdentityRepository> = Arc::new(RwLock::new(HashMap::new()));

    // Replace hash maps with factory methods here
    let schemas_repository: Arc<SchemaRepository> = Arc::new(RwLock::new(HashMap::new()));
    let entities_repository: Arc<PrincipalsRepository> = Arc::new(RwLock::new(HashMap::new()));
    let principal_association_repository: Arc<PrincipalAssociationRepository> = Arc::new(RwLock::new(HashMap::new()));

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
            .app_data(Data::new(policy_repository.clone()))
            .app_data(Data::new(policy_attachments_repository.clone()))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            .app_data(Data::new(principal_association_repository.clone()))
            // Token endpoint
            .service(token)
            .service(policy::crud())
            .service(identity::crud())
            .service(attachment::crud())
            .service(schema::crud())
            .service(principal::crud())
            .service(association::crud())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
}
