use actix_web::get;
use actix_web::web::{Data, Path, ReqData};
use boxer_core::models::external_token::ExternalToken;
use boxer_core::services::token_service::internal_token_service::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use boxer_core::services::token_service::TokenService;
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
    token_service: Data<Arc<dyn TokenService>>,
    identity_provider: Path<String>,
) -> actix_web::Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    token_service
        .into_inner()
        .issue_internal_token(ip, external_token.into_inner())
        .await
        .map_err(|err| {
            error!("Failed to issue internal token: {}", err);
            actix_web::error::ErrorUnauthorized("Unauthorized")
        })
}
