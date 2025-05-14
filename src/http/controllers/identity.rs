use crate::http::errors::*;
use crate::models::external::identity::ExternalIdentity;
use crate::services::base::upsert_repository::IdentityRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(context_path = "/identity/", responses((status = OK)))]
#[post("{identity_provider}/{id}")]
pub async fn post(params: Path<(String, String)>, data: Data<Arc<IdentityRepository>>) -> Result<HttpResponse> {
    let key = params.into_inner();
    let eid = ExternalIdentity::from(key.clone());
    data.upsert(key, eid).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/identity/", responses((status = OK)))]
#[get("{identity_provider}/{id}")]
pub async fn get(params: Path<(String, String)>, data: Data<Arc<IdentityRepository>>) -> Result<impl Responder> {
    let eid = data.get(params.into_inner()).await?;
    Ok(web::Json(eid))
}

#[utoipa::path(context_path = "/identity/", responses((status = OK)))]
#[delete("{identity_provider}/{id}")]
pub async fn delete(params: Path<(String, String)>, data: Data<Arc<IdentityRepository>>) -> Result<HttpResponse> {
    data.delete(params.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity").service(post).service(get).service(delete)
}
