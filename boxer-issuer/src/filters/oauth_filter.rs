use std::future::{ready, Ready};
use std::rc::Rc;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::error::ErrorUnauthorized;
use futures_util::future::LocalBoxFuture;
use crate::services::external_identity_validator::{ExternalIdentityValidator,ExternalIdentityValidatorImpl};

// Middleware for external token validation factory
pub struct ExternalTokenMiddlewareFactory {
    
}

// The ExternalTokenMiddlewareFactory's own methods implementation
impl ExternalTokenMiddlewareFactory {
    pub(crate) fn new() -> Self {
        ExternalTokenMiddlewareFactory{}
    }
    
    pub(crate) fn create(&self) -> Rc<dyn  ExternalIdentityValidator> {
        Rc::new(ExternalIdentityValidatorImpl{})
    }
}

// Transform trait implementation
// `NextServiceType` - type of the next service
// `BodyType` - type of response's body
impl<NextServiceType, BodyType> Transform<NextServiceType, ServiceRequest> for ExternalTokenMiddlewareFactory
where
    NextServiceType: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextServiceType::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Transform = OauthMiddleware<NextServiceType>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: NextServiceType) -> Self::Future {
        let validator = self.create();
        let mw = OauthMiddleware {
            service: Rc::new(service), external_identity_validator: validator
        };
        ready(Ok(mw))
    }
}

pub struct OauthMiddleware<NextServiceType> {
    service: Rc<NextServiceType>,
    external_identity_validator: Rc<dyn ExternalIdentityValidator>,
}

impl<NextServiceType, BodyType> Service<ServiceRequest> for OauthMiddleware<NextServiceType>
where
    NextServiceType: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextServiceType::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let validator = Rc::clone(&self.external_identity_validator);
        Box::pin(async move {
            let validation_result = validator.validate("token").await;
            if  validation_result.is_err() {
                return Err(ErrorUnauthorized("Unauthorized"));
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
