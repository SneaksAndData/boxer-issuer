use crate::http::errors::*;
use crate::models::external::identity::{ExternalIdentity, PolicyAttachment};
use crate::services::base::upsert_repository::PolicyAttachmentRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(context_path = "/attachment/", responses((status = OK)))]
#[post("{identity_provider}/{id}/{policy_id}")]
pub async fn post(
    params: Path<(String, String, String)>,
    data: Data<Arc<PolicyAttachmentRepository>>,
) -> Result<HttpResponse> {
    let (identity_provider, id, policy_id) = params.into_inner();
    let eid = ExternalIdentity::new(identity_provider, id);
    let attachment = PolicyAttachment::single(policy_id);
    data.upsert(eid, attachment).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/attachment/", responses((status = OK)))]
#[get("{identity_provider}/{id}")]
pub async fn get(
    params: Path<(String, String)>,
    data: Data<Arc<PolicyAttachmentRepository>>,
) -> Result<impl Responder> {
    let (identity_provider, id) = params.into_inner();
    let eid = ExternalIdentity::new(identity_provider, id);
    let result = data.get(eid).await?;
    Ok(web::Json(result))
}

#[utoipa::path(context_path = "/attachment/", responses((status = OK)))]
#[delete("{identity_provider}/{id}")]
pub async fn delete(
    params: Path<(String, String)>,
    data: Data<Arc<PolicyAttachmentRepository>>,
) -> Result<HttpResponse> {
    let (identity_provider, id) = params.into_inner();
    let eid = ExternalIdentity::new(id, identity_provider);
    data.delete(eid).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/attachment").service(post).service(get).service(delete)
}
