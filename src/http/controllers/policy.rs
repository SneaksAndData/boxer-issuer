use crate::http::errors::*;
use crate::models::api::external::identity::Policy;
use crate::services::base::upsert_repository::PolicyRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Path};
use actix_web::{delete, get, post, web, HttpResponse};
use std::sync::Arc;

#[utoipa::path(context_path = "/policy/", responses((status = OK)))]
#[post("{id}")]
async fn post(id: Path<String>, policy: String, data: Data<Arc<PolicyRepository>>) -> Result<HttpResponse> {
    data.upsert(id.to_string(), Policy::new(policy)).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/policy/", responses((status = OK)))]
#[get("{id}")]
async fn get(id: Path<String>, data: Data<Arc<PolicyRepository>>) -> Result<String> {
    let policy = data.get(id.to_string()).await?;
    Ok(policy.content)
}

#[utoipa::path(context_path = "/policy/", responses((status = OK)))]
#[delete("{id}")]
async fn delete(id: Path<String>, data: Data<Arc<PolicyRepository>>) -> Result<HttpResponse> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/policy").service(post).service(get).service(delete)
}
