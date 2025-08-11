pub mod external_identity_registration;

use crate::http::controllers::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use std::sync::Arc;

#[utoipa::path(context_path = "/identity/", responses((status = OK)))]
#[post("")]
pub async fn post_identity(
    request: Json<ExternalIdentityRegistration>,
    data: Data<Arc<IdentityRepository>>,
) -> Result<HttpResponse> {
    let key = (request.identity_provider.clone(), request.user_id.clone());
    data.upsert(key, request.into_inner()).await?;
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
