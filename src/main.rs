mod http;
mod models;
mod services;

use crate::http::urls::{
    delete_identity, delete_policy, delete_policy_attachment, get_identity, get_policy,
    get_policy_attachment, post_identity, post_policy, post_policy_attachment, token,
};
use crate::services::base::upsert_repository::{
    IdentityRepository, PolicyAttachmentRepository, PolicyRepository,
};
use crate::services::configuration_manager::ConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::token_service::TokenService;
use actix_web::{web, App, HttpServer};
use log::info;
use std::collections::HashMap;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8080);
    let validator_provider = Arc::new(identity_validator_provider::new());
    let cm = Arc::clone(&validator_provider);
    let secret = Arc::new(cm.get_signing_key());

    let _ = tokio::spawn(cm.watch_for_identity_providers());
    info!("Configuration manager started");

    let policy_repository: Arc<PolicyRepository> = Arc::new(RwLock::new(HashMap::new()));
    let policy_attachments_repository: Arc<PolicyAttachmentRepository> =
        Arc::new(RwLock::new(HashMap::new()));
    let identity_repository: Arc<IdentityRepository> = Arc::new(RwLock::new(HashMap::new()));

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        let token_provider = Arc::new(TokenService::new(
            validator_provider.clone(),
            policy_repository.clone(),
            policy_attachments_repository.clone(),
            Arc::clone(&secret),
        ));
        App::new()
            // Application services
            .app_data(web::Data::new(token_provider))
            .app_data(web::Data::new(policy_repository.clone()))
            .app_data(web::Data::new(policy_attachments_repository.clone()))
            .app_data(web::Data::new(identity_repository.clone()))
            // Token endpoint
            .service(token)
            // Policy CRUD
            .service(post_policy)
            .service(get_policy)
            .service(delete_policy)
            // Identity CRUD
            .service(post_identity)
            .service(get_identity)
            .service(delete_identity)
            // Policy Attachment CRUD
            .service(post_policy_attachment)
            .service(get_policy_attachment)
            .service(delete_policy_attachment)
    })
    .bind(addr)?
    .run()
    .await
}
