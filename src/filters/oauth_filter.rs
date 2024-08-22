use crate::services::external_identity_validator::{
    ExternalIdentityValidator, ExternalIdentityValidatorImpl,
};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;

// Middleware for external token validation factory
pub struct ExternalTokenMiddlewareFactory {}

// The ExternalTokenMiddlewareFactory's own methods implementation
impl ExternalTokenMiddlewareFactory {
    pub(crate) fn new() -> Self {
        ExternalTokenMiddlewareFactory {}
    }

    pub(crate) fn create(&self) -> Arc<dyn ExternalIdentityValidator> {
        Arc::new(ExternalIdentityValidatorImpl {})
    }
}

// Transform trait implementation
// `NextServiceType` - type of the next service
// `BodyType` - type of response's body
impl<NextService, BodyType> Transform<NextService, ServiceRequest>
    for ExternalTokenMiddlewareFactory
where
    NextService:
        Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextService::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Transform = OauthMiddleware<NextService>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: NextService) -> Self::Future {
        let validator = self.create();
        let mw = OauthMiddleware {
            service: Arc::new(service),
            external_identity_validator: validator,
        };
        ready(Ok(mw))
    }
}

// The middleware object
pub struct OauthMiddleware<NextService> {
    service: Arc<NextService>,
    external_identity_validator: Arc<dyn ExternalIdentityValidator>,
}

// The middleware implementation
impl<NextService, BodyType> Service<ServiceRequest> for OauthMiddleware<NextService>
where
    NextService:
        Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextService::Future: 'static,
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
        let validator = Arc::clone(&self.external_identity_validator);

        // The async block that will be executed when the middleware is called
        Box::pin(async move {
            let validation_result = validator.validate("token").await;
            if validation_result.is_err() {
                return Err(ErrorUnauthorized("Unauthorized"));
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
