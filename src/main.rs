mod http;
mod models;
mod services;

use crate::http::urls::token;
use crate::services::configuration_manager::ConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::policy_repository::InMemoryPolicyRepository;
use crate::services::token_service::TokenService;
use actix_web::{web, App, HttpServer};
use log::info;
use std::io::Result;
use std::sync::Arc;

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

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        let policy_repository = Arc::new(InMemoryPolicyRepository::new());
        let validator_provider = Arc::clone(&validator_provider);
        let token_provider = Arc::new(TokenService::new(
            validator_provider,
            policy_repository,
            Arc::clone(&secret),
        ));
        App::new()
            .app_data(web::Data::new(token_provider))
            .service(token)
    })
    .bind(addr)?
    .run()
    .await
}
