mod http;
mod models;
mod services;

use crate::http::urls::{
    delete_identity, delete_policy, delete_policy_attachment, get_identity, get_policy, get_policy_attachment,
    post_identity, post_policy, post_policy_attachment, token,
};
use crate::services::base::upsert_repository::{IdentityRepository, PolicyAttachmentRepository, PolicyRepository};
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

use crate::http::urls;
use utoipa_actix_web::AppExt;

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

    #[derive(OpenApi)]
    #[openapi(paths(
        urls::token,
        urls::post_policy,
        urls::get_policy,
        urls::delete_policy,
        urls::post_identity,
        urls::get_identity,
        urls::delete_identity,
        urls::post_policy_attachment,
        urls::get_policy_attachment,
        urls::delete_policy_attachment,
    ))]
    struct ApiDoc;

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        let token_provider = Arc::new(TokenService::new(
            validator_provider.clone(),
            policy_repository.clone(),
            policy_attachments_repository.clone(),
            Arc::clone(&secret),
        ));
        App::new()
            .into_utoipa_app()
            // Application services
            .app_data(Data::new(token_provider))
            .app_data(Data::new(policy_repository.clone()))
            .app_data(Data::new(policy_attachments_repository.clone()))
            .app_data(Data::new(identity_repository.clone()))
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
            // Swagger UI
            .into_app()
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
}
