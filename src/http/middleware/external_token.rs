mod token_not_present_error;
mod with_external_token_id;

use crate::http::middleware::external_token::token_not_present_error::ExternalTokenError;
use crate::models::api::external::token::ExternalToken;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use with_external_token_id::WithExternalTokenId;

/// Middleware to initialize the audit chain for incoming requests.
/// This should be the first middleware in the audit chain to ensure that all subsequent middleware
/// and handlers have access to the audit context.
pub async fn external_token<Request: WithExternalTokenId, Error: ExternalTokenError>(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let header_value = request.headers().get("Authorization");

    match header_value {
        None => Err(Error::external_token_not_present(&request).into()),
        Some(header_value) => {
            let token =
                ExternalToken::try_from(header_value).map_err(|e| Error::token_extraction_failed(&request, e))?;
            next.call(Request::from(request).with_external_token_id(token)).await
        }
    }
}
