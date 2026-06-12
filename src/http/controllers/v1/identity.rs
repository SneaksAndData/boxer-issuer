pub mod external_identity_registration;
mod external_identity_registration_request;

use crate::http::controllers::v1::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::http::controllers::v1::identity::external_identity_registration_request::ExternalIdentityRegistrationRequest;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use cedar_policy::EntityUid;
use std::str::FromStr;
use std::sync::Arc;

#[utoipa::path(context_path = "/identity/",
    request_body = ExternalIdentityRegistrationRequest,
    responses(
        (status = OK)
    ),
    security(
        ("internal" = [])
    )
)]
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

#[utoipa::path(context_path = "/identity/",
    responses(
        (status = OK, body = ExternalIdentityRegistration),
        (status = NOT_FOUND, description = "Identity does not exist")
    ),
    security(
        ("internal" = [])
    )
)]
#[get("{identity_provider}/{id}")]
pub async fn get_identity(
    params: Path<(String, String)>,
    data: Data<Arc<IdentityRepository>>,
) -> Result<impl Responder> {
    let eid: ExternalIdentityRegistration = data.get(params.into_inner()).await?;
    Ok(Json(eid))
}

#[utoipa::path(context_path = "/identity/",
    responses(
        (status = OK)
    ),
    security(
        ("internal" = [])
    )
)]
#[delete("{identity_provider}/{id}")]
pub async fn delete_identity(
    params: Path<(String, String)>,
    data: Data<Arc<IdentityRepository>>,
    principal_data: Data<Arc<PrincipalRepository>>,
) -> Result<HttpResponse> {
    let identity_provider_id = params.into_inner();
    let identity = data.get(identity_provider_id.clone()).await?;
    let pid = EntityUid::from_str(&identity.principal_id).map_err(actix_web::error::ErrorBadRequest)?;
    let principal_identity = PrincipalIdentity::new(identity.principal_schema, pid);
    principal_data.delete(principal_identity).await?;
    data.delete(identity_provider_id.clone()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/identity")
        .service(post_identity)
        .service(get_identity)
        .service(delete_identity)
}
