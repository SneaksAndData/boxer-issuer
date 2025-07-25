use crate::http::errors::*;
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::base::upsert_repository::IdentityProviderRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Data;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use anyhow::anyhow;
use std::sync::Arc;

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK)))]
#[post("identity_provider/{id}")]
pub async fn post(
    id: String,
    identity_provider_json: String,
    data: Data<Arc<IdentityProviderRepository>>,
) -> Result<HttpResponse> {
    let registration: IdentityProviderRegistration =
        serde_json::from_str(&identity_provider_json).map_err(|e| anyhow!("Failed to parse registration. {}", e))?;
    data.upsert(id, registration).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK)))]
#[get("identity_provider/{id}")]
pub async fn get(id: String, data: Data<Arc<IdentityProviderRepository>>) -> Result<impl Responder> {
    let eid = data.get(id).await?;
    Ok(web::Json(eid))
}

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK)))]
#[delete("identity_provider/{id}")]
pub async fn delete(id: String, data: Data<Arc<IdentityProviderRepository>>) -> Result<HttpResponse> {
    data.delete(id).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity_provider")
        .service(post)
        .service(get)
        .service(delete)
}
