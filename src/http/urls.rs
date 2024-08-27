use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::{error, get, web, HttpRequest};
use log::error;
use std::sync::Arc;

#[get("/token/{identity_provider}")]
pub async fn token(
    data: web::Data<Arc<TokenService>>,
    identity_provider: web::Path<String>,
    req: HttpRequest,
) -> actix_web::Result<String> {
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
        None => Err(error::ErrorUnauthorized("No Authorization header found")),
    }
}
