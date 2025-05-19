mod http;
mod models;
mod services;

use crate::http::controllers::{attachment, identity, policy, principal, schema, token::token};
use crate::services::base::upsert_repository::{
    IdentityRepository, PolicyAttachmentRepository, PolicyRepository, PrincipalsRepository, SchemaRepository,
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

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        let token_provider = Arc::new(TokenService::new(
            validator_provider.clone(),
            policy_repository.clone(),
            policy_attachments_repository.clone(),
            Arc::clone(&secret),
        ));
        App::new()
            .app_data(Data::new(token_provider))
            .app_data(Data::new(policy_repository.clone()))
            .app_data(Data::new(policy_attachments_repository.clone()))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            // Token endpoint
            .service(token)
            .service(policy::crud())
            .service(identity::crud())
            .service(attachment::crud())
            .service(schema::crud())
            .service(principal::crud())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
}
