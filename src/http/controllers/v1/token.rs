use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::get;
use actix_web::web::{Data, Path, ReqData};
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
    external_token: ReqData<ExternalToken>,
    data: Data<Arc<TokenService>>,
    identity_provider: Path<String>,
) -> actix_web::Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let internal_token = data.issue_token(ip, external_token.into_inner()).await.map_err(|err| {
        error!("Token issuance failed: {}", err);
        actix_web::error::ErrorUnauthorized("Unauthorized")
    })?;
    Ok(internal_token)
}
