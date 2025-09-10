use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::web::{Data, Path};
use actix_web::{get, HttpRequest};
use log::error;
use std::sync::Arc;

#[utoipa::path(
    responses((status = OK, body = String)),
    security(
        ("external" = [])
    )
)]
#[get("/token/{identity_provider}")]
pub async fn token(
    data: Data<Arc<TokenService>>,
    identity_provider: Path<String>,
    req: HttpRequest,
) -> actix_web::Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());

    let header = req.headers().get("Authorization").ok_or_else(|| {
        error!("Authorization header missing");
        actix_web::error::ErrorUnauthorized("Unauthorized")
    })?;

    let token = ExternalToken::try_from(header).map_err(|err| {
        error!("Invalid token format: {}", err);
        actix_web::error::ErrorUnauthorized("Unauthorized")
    })?;
    let internal_token = data.issue_token(ip, token).await.map_err(|err| {
        error!("Token issuance failed: {}", err);
        actix_web::error::ErrorUnauthorized("Unauthorized")
    })?;
    Ok(internal_token)
}
