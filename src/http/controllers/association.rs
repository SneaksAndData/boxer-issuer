use crate::http::errors::*;
use crate::models::api::external::identity::ExternalIdentity;
use crate::services::base::upsert_repository::{PrincipalAssociationRepository, PrincipalIdentity};
use crate::services::principal_service::{IdentityAssociationRequest, PrincipalService};
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
struct IdentityAssociation {
    identity_provider: String,
    identity: String,
    principal_schema: String,
    principal_id: String,
}

#[utoipa::path(context_path = "/association/", responses((status = OK)))]
#[post("/")]
async fn post(
    request: Json<IdentityAssociation>,
    principal_service: Data<Arc<PrincipalService>>,
) -> Result<HttpResponse> {
    let request = IdentityAssociationRequest {
        external_identity_info: (request.identity_provider.clone(), request.identity.clone()),
        principal_id: PrincipalIdentity::from((request.principal_schema.clone(), request.principal_id.clone())),
    };
    principal_service.associate(request).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    context_path = "/association/identities",
    responses(
        (status = OK, body = IdentityAssociation)
    )
)]
#[get("/identities/{identity_provider}/{id}")]
async fn get(path: Path<(String, String)>, data: Data<Arc<PrincipalAssociationRepository>>) -> Result<impl Responder> {
    let external_identity = ExternalIdentity::from(path.into_inner());
    let principal_id = data.get(external_identity.clone()).await?;
    Ok(Json(IdentityAssociation {
        identity_provider: external_identity.identity_provider,
        identity: external_identity.user_id,
        principal_schema: principal_id.schema_id().clone(),
        principal_id: principal_id.principal_id().clone(),
    }))
}

#[utoipa::path(context_path = "/association/identities", responses((status = OK)))]
#[get("/identities/{identity_provider}/{id}")]
async fn delete(path: Path<(String, String)>, data: Data<Arc<PrincipalAssociationRepository>>) -> Result<HttpResponse> {
    let external_identity = ExternalIdentity::from(path.into_inner());
    data.delete(external_identity).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/association").service(post).service(get).service(delete)
}
