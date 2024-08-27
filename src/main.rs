mod services;
mod models;
mod http;

use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::configuration_manager::ConfigurationManager;
use crate::services::identity_validator_provider;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::{error, get, web, App, HttpRequest, HttpServer};
use std::io::Result;
use std::sync::Arc;
use log::{error, info};

#[get("/token/{identity_provider}")]
async fn token(data: web::Data<Arc<TokenService>>, identity_provider: web::Path<String>, req: HttpRequest) -> actix_web::Result<String>
{
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let maybe_header = req.headers().get("Authorization");
    match maybe_header {
        Some(header) => {
            let token = ExternalToken::try_from(header).map_err(|e| {
                error!("Error: {:?}", e);
                error::ErrorUnauthorized("Invalid token format")
            })?;
            data.issue_token(ip, token).await.map_err(|e| {
                error!("Error: {:?}", e);
                error::ErrorUnauthorized("Internal Server Error")
            })
        }
        None => Err(error::ErrorUnauthorized("No Authorization header found"))
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = ("127.0.0.1", 8080);
    
    let ref_counter= Arc::new(identity_validator_provider::new());
    
    let cm = Arc::clone(&ref_counter);
    let _ = tokio::spawn(cm.watch_for_identity_providers());
    info!("Configuration manager started");

    info!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new( move || {
        let token_provider = Arc::new(TokenService::new(Arc::clone(&ref_counter)));
        App::new().app_data(web::Data::new(token_provider)).service(token)
    })
    .bind(addr)?
    .run()
    .await
}
