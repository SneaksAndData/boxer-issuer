use crate::http::errors::*;
use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::base::upsert_repository::IdentityProviderRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
struct OidcIdentityProviderRegistration {
    pub user_id_claim: String,
    pub discovery_url: String,
    pub issuers: Vec<String>,
    pub audiences: Vec<String>,
}

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK)))]
#[post("oidc/{id}")]
pub async fn post(
    id: Path<String>,
    registration: Json<OidcIdentityProviderRegistration>,
    data: Data<Arc<IdentityProviderRepository>>,
) -> Result<HttpResponse> {
    let registration = IdentityProviderRegistration {
        name: id.clone(),
        oidc: OidcExternalIdentityProviderSettings {
            user_id_claim: registration.user_id_claim.clone(),
            discovery_url: registration.discovery_url.clone(),
            issuers: registration.issuers.clone(),
            audiences: registration.audiences.clone(),
        },
    };
    data.upsert(id.into_inner(), registration).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK, body = OidcIdentityProviderRegistration)))]
#[get("oidc/{id}")]
pub async fn get(id: Path<String>, data: Data<Arc<IdentityProviderRepository>>) -> Result<impl Responder> {
    let eid = data.get(id.into_inner()).await?;
    Ok(web::Json(eid))
}

#[utoipa::path(context_path = "/identity_provider/", responses((status = OK)))]
#[delete("oidc/{id}")]
pub async fn delete(id: Path<String>, data: Data<Arc<IdentityProviderRepository>>) -> Result<HttpResponse> {
    data.delete(id.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity_provider")
        .service(post)
        .service(get)
        .service(delete)
}
