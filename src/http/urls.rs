use crate::models::external::identity::{ExternalIdentity, Policy, PolicyAttachment};
use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::base::upsert_repository::{
    IdentityRepository, PolicyAttachmentRepository, PolicyRepository,
};
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::{delete, error, get, post, web, HttpRequest, HttpResponse, Responder};
use log::error;
use std::sync::Arc;

#[get("/token/{identity_provider}")]
pub async fn token(
    data: web::Data<Arc<TokenService>>,
    identity_provider: web::Path<String>,
    req: HttpRequest,
) -> actix_web::Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let maybe_header = req.headers().get("Authorization");
    match maybe_header {
        Some(header) => {
            let token = ExternalToken::try_from(header).map_err(|e| {
                error!("Error: {:?}", e);
                error::ErrorUnauthorized("Invalid token format")
            })?;
            data.issue_token(ip, token).await.map_err(|e| {
                error!("Error: {:?}", e);
                error::ErrorUnauthorized("Internal Server Error")
            })
        }
        None => Err(error::ErrorUnauthorized("No Authorization header found")),
    }
}

#[post("/policy/{id}")]
pub async fn post_policy(
    id: web::Path<String>,
    policy: String,
    data: web::Data<Arc<PolicyRepository>>,
) -> actix_web::Result<HttpResponse> {
    data.upsert(id.to_string(), Policy::new(policy))
        .await
        .map_err(|e| {
            error!("Error: {:?}", e);
            error::ErrorInternalServerError("Failed to upsert policy")
        })?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/policy/{id}")]
pub async fn get_policy(
    id: web::Path<String>,
    data: web::Data<Arc<PolicyRepository>>,
) -> actix_web::Result<String> {
    let policy = data.get(id.to_string()).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(policy.content)
}

#[delete("/policy/{id}")]
pub async fn delete_policy(
    id: web::Path<String>,
    data: web::Data<Arc<PolicyRepository>>,
) -> actix_web::Result<HttpResponse> {
    data.delete(id.to_string()).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/identity/{identity_provider}/{id}")]
pub async fn post_identity(
    params: web::Path<(String, String)>,
    data: web::Data<Arc<IdentityRepository>>,
) -> actix_web::Result<HttpResponse> {
    let key = params.into_inner();
    let eid = ExternalIdentity::from(key.clone());
    data.upsert(key, eid).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/identity/{identity_provider}/{id}")]
pub async fn get_identity(
    params: web::Path<(String, String)>,
    data: web::Data<Arc<IdentityRepository>>,
) -> actix_web::Result<impl Responder> {
    let eid = data.get(params.into_inner()).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(web::Json(eid))
}

#[delete("/identity/{identity_provider}/{id}")]
pub async fn delete_identity(
    params: web::Path<(String, String)>,
    data: web::Data<Arc<IdentityRepository>>,
) -> actix_web::Result<HttpResponse> {
    data.delete(params.into_inner()).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/attachment/{identity_provider}/{id}/{policy_id}")]
pub async fn post_policy_attachment(
    params: web::Path<(String, String, String)>,
    data: web::Data<Arc<PolicyAttachmentRepository>>,
) -> actix_web::Result<HttpResponse> {
    let (identity_provider, id, policy_id) = params.into_inner();
    let eid = ExternalIdentity::new(id, identity_provider);
    let attachment = PolicyAttachment::single(policy_id);
    data.upsert(eid, attachment).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/attachment/{identity_provider}/{id}")]
pub async fn get_policy_attachment(
    params: web::Path<(String, String)>,
    data: web::Data<Arc<PolicyAttachmentRepository>>,
) -> actix_web::Result<impl Responder> {
    let (identity_provider, id) = params.into_inner();
    let eid = ExternalIdentity::new(id, identity_provider);
    let result = data.get(eid).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(web::Json(result))
}

#[delete("/attachment/{identity_provider}/{id}")]
pub async fn delete_policy_attachment(
    params: web::Path<(String, String)>,
    data: web::Data<Arc<PolicyAttachmentRepository>>,
) -> actix_web::Result<HttpResponse> {
    let (identity_provider, id) = params.into_inner();
    let eid = ExternalIdentity::new(id, identity_provider);
    data.delete(eid).await.map_err(|e| {
        error!("Error: {:?}", e);
        error::ErrorInternalServerError("Failed to upsert policy")
    })?;
    Ok(HttpResponse::Ok().finish())
}
