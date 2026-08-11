use actix_web::error::ErrorInternalServerError;
use actix_web::get;
use actix_web::web::{Data, Path, ReqData};
use boxer_core::models::external_token::ExternalToken;
use boxer_core::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
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
    audit_event: ReqData<AuditEvent>,
) -> actix_web::Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let audit_event = audit_event.into_inner();
    let token_audit_event = match audit_event {
        AuditEvent::Intermediate(IntermediateAuditEvent {
            external_token: Some(token_event),
            ..
        }) => Ok(token_event),
        _ => Err(ErrorInternalServerError(format!(
            "Unexpected audit event: {:?}",
            audit_event
        ))),
    }?;
    token_service
        .into_inner()
        .issue_internal_token(ip, external_token.into_inner(), token_audit_event)
        .await
        .map_err(|err| {
            error!("Failed to issue internal token: {}", err);
            actix_web::error::ErrorUnauthorized("Unauthorized")
        })
}
