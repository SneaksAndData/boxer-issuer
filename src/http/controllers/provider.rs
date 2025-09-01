mod oidc_provider_registration;

use crate::http::controllers::provider::oidc_provider_registration::OidcIdentityProviderRegistration;
use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use std::sync::Arc;

#[utoipa::path(context_path = "/identity_provider/",
    responses(
        (status = OK),
    ),
    security(
        ("internal" = [])
    )
)]
#[post("oidc/{id}")]
pub async fn post_provider(
    id: Path<String>,
    registration: Json<OidcIdentityProviderRegistration>,
    data: Data<Arc<IdentityProviderRepository>>,
) -> Result<HttpResponse> {
    let registration = IdentityProviderRegistration {
        name: id.clone(),
        oidc: Some(OidcExternalIdentityProviderSettings {
            user_id_claim: registration.user_id_claim.clone(),
            discovery_url: registration.discovery_url.clone(),
            issuers: registration.issuers.clone(),
            audiences: registration.audiences.clone(),
        }),
    };
    data.upsert(id.into_inner(), registration).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/identity_provider/",
    responses(
        (status = OK, body = OidcIdentityProviderRegistration),
        (status = NOT_FOUND, description = "Identity provider does not exist")
    ),
    security(
        ("internal" = [])
    )
)]
#[get("oidc/{id}")]
pub async fn get_provider(id: Path<String>, data: Data<Arc<IdentityProviderRepository>>) -> Result<impl Responder> {
    let eid = data.get(id.into_inner()).await?;
    Ok(Json(eid.oidc))
}
#[utoipa::path(context_path = "/identity_provider/",
    responses(
        (status = OK)
    ),
    security(
        ("internal" = [])
    )
)]
#[delete("oidc/{id}")]
pub async fn delete_provider(id: Path<String>, data: Data<Arc<IdentityProviderRepository>>) -> Result<HttpResponse> {
    data.delete(id.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity_provider")
        .service(post_provider)
        .service(get_provider)
        .service(delete_provider)
}
