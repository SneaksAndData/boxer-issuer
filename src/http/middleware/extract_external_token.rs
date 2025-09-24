use crate::models::api::external::token::ExternalToken;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use boxer_core::services::audit::events::token_validation_event::TokenValidationEvent;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::audit::AuditService;
use boxer_core::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use futures_util::future::LocalBoxFuture;
use log::{error, warn};
use opentelemetry::context::FutureExt;
use std::collections::HashSet;
use std::sync::Arc;

/// Middleware for external token validation factory
pub struct ExternalTokenMiddlewareFactory {
    pub audit_service: Arc<dyn AuditService>,
}

/// The ExternalTokenMiddlewareFactory's own methods implementation
impl ExternalTokenMiddlewareFactory {
    pub(crate) fn new(audit_service: Arc<dyn AuditService>) -> Self {
        ExternalTokenMiddlewareFactory { audit_service }
    }
}

/// Default implementation for ExternalTokenMiddlewareFactory that uses LogAuditService
impl Default for ExternalTokenMiddlewareFactory {
    fn default() -> Self {
        Self::new(Arc::new(LogAuditService::new()))
    }
}

/// Transform trait implementation
/// `NextServiceType` - type of the next service
/// `BodyType` - type of response's body
impl<NextService, BodyType> Transform<NextService, ServiceRequest> for ExternalTokenMiddlewareFactory
where
    NextService: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextService::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Transform = ExternalTokenMiddleware<NextService>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<ExternalTokenMiddleware<NextService>, Self::InitError>>;

    fn new_transform(&self, service: NextService) -> Self::Future {
        let audit_service = self.audit_service.clone();
        Box::pin(async move {
            let mw = ExternalTokenMiddleware {
                service: Arc::new(service),
                audit_service,
            };
            Ok(mw)
        })
    }
}

/// The middleware object
pub struct ExternalTokenMiddleware<NextService> {
    service: Arc<NextService>,
    pub audit_service: Arc<dyn AuditService>,
}

impl<Next> ExternalTokenMiddleware<Next> {
    fn get_token(req: &ServiceRequest) -> Result<ExternalToken, Error> {
        let token_value = req
            .headers()
            .get("Authorization")
            .ok_or(ErrorUnauthorized("Unauthorized"))?;
        ExternalToken::try_from(token_value).map_err(|err| {
            error!("Invalid token format: {}", err);
            ErrorUnauthorized("Unauthorized")
        })
    }
}

/// The middleware implementation
impl<Next, BodyType> Service<ServiceRequest> for ExternalTokenMiddleware<Next>
where
    Next: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    Next::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    // Asynchronously handle the request and bypass it to the next service
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone the service and validator to be able to use them in the async block
        let service = Arc::clone(&self.service);

        let audit_service = Arc::clone(&self.audit_service);

        let future = async move {
            let parent = start_trace("external_token_validation", None);
            let external_token_result = Self::get_token(&req);

            match external_token_result {
                Ok(external_token) => {
                    let event = TokenValidationEvent::external(&external_token.token, true, HashSet::new());
                    // make nested block to avoid borrowing issues
                    {
                        let mut ext = req.extensions_mut();
                        ext.insert(external_token);
                    }
                    audit_service
                        .record_token_validation(event)
                        .map_err(ErrorUnauthorized)?;
                    let res = service
                        .call(req)
                        .with_context(parent.clone())
                        .await
                        .stop_trace(parent)?;
                    Ok(res)
                }
                Err(err) => {
                    warn!("Failed to extract external token: {}", err);
                    let mut details = HashSet::new();
                    details.insert(err.to_string());
                    let event = TokenValidationEvent::external_empty(false, details);
                    audit_service
                        .record_token_validation(event)
                        .map_err(ErrorUnauthorized)?;
                    Err(ErrorUnauthorized("Unauthorized")).stop_trace(parent)?
                }
            }
        };
        Box::pin(future)
    }
}
