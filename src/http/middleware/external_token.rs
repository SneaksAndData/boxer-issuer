mod with_external_token_id;

use crate::models::api::external::token::ExternalToken;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::Error;
use boxer_core::http::middleware::audit::audited_error::AuditedError;
use with_external_token_id::AuditExternalToken;

/// Middleware to initialize the audit chain for incoming requests.
/// This should be the first middleware in the audit chain to ensure that all subsequent middleware
/// and handlers have access to the audit context.
pub async fn external_token<Request: AuditExternalToken>(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let token = ExternalToken::try_from(&req).map_err(|e: anyhow::Error| AuditedError::wrap_err(&req, e))?;
    next.call(Request::from(req).with_external_token_id(token)).await
}
