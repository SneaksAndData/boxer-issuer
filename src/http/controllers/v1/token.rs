use crate::http::errors::*;
use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::web::{Data, Path};
use actix_web::{get, HttpRequest};
use boxer_core::services::audit::events::token_validation_event::{TokenValidationEvent, TokenValidationResult};
use boxer_core::services::audit::AuditService;
use std::sync::Arc;

#[utoipa::path(
    responses((status = OK, body = String)),
    security(
        ("external" = [])
    )
)]
#[get("/token/{identity_provider}")]
pub async fn token(
    audit_service: Data<Arc<dyn AuditService>>,
    data: Data<Arc<TokenService>>,
    identity_provider: Path<String>,
    req: HttpRequest,
) -> Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let maybe_token = req.headers().get("Authorization");
    let maybe_header = maybe_token.clone().map(|header| async move {
        let token = ExternalToken::try_from(header)?;
        data.issue_token(ip, token).await
    });

    let result = match maybe_header {
        Some(fut) => {
            let token = fut.await?;
            Ok(token)
        }
        None => Err(Error::new("Unauthorized")),
    };

    let audit_result = match &result {
        Ok(_) => TokenValidationResult::Success,
        Err(ref err) => TokenValidationResult::Failure(err.to_string()),
    };

    let token_hash = maybe_token
        .map(|header| format!("{:x}", md5::compute(header)))
        .unwrap_or("empty".to_string());
    let event = TokenValidationEvent::external(token_hash, audit_result);
    audit_service.record_token_validation(event).map_err(|err| {
        log::error!("Failed to record audit event: {}", err);
        Error::new("Unauthorized")
    })?;
    result
}
