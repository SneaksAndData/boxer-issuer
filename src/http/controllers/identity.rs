pub mod external_identity_registration;
mod external_identity_registration_request;

use crate::http::controllers::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::http::controllers::identity::external_identity_registration_request::ExternalIdentityRegistrationRequest;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use std::sync::Arc;

#[utoipa::path(context_path = "/identity/", request_body = ExternalIdentityRegistrationRequest, responses((status = OK)))]
#[post("{identity_provider}/{id}")]
pub async fn post_identity(
    params: Path<(String, String)>,
    request: Json<ExternalIdentityRegistrationRequest>,
    data: Data<Arc<IdentityRepository>>,
) -> Result<HttpResponse> {
    let (identity_provider, id) = params.into_inner();
    let key = (identity_provider.clone(), id.clone());
    let registration = ExternalIdentityRegistration::from_request(identity_provider, id, request.into_inner());
    data.upsert(key, registration).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/identity/", responses((status = OK, body = ExternalIdentityRegistration)))]
#[get("{identity_provider}/{id}")]
pub async fn get_identity(
    params: Path<(String, String)>,
    data: Data<Arc<IdentityRepository>>,
) -> Result<impl Responder> {
    let eid: ExternalIdentityRegistration = data.get(params.into_inner()).await?;
    Ok(Json(eid))
}

#[utoipa::path(context_path = "/identity/", responses((status = OK)))]
#[delete("{identity_provider}/{id}")]
pub async fn delete_identity(
    params: Path<(String, String)>,
    data: Data<Arc<IdentityRepository>>,
) -> Result<HttpResponse> {
    data.delete(params.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity")
        .service(post_identity)
        .service(get_identity)
        .service(delete_identity)
}
