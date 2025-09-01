mod principal_create_response;

use crate::http::controllers::principal::principal_create_response::PrincipalCreateResponse;
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, web, Responder, Result};
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use cedar_policy::{Entity, EntityUid, Schema};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;

#[utoipa::path(context_path = "/principal/",
    request_body = Value,
    responses(
        (status = OK, body = PrincipalCreateResponse),
    ),
    security(
        ("internal" = [])
    )
)]
#[post("{schema}")]
async fn post_principal(
    schema_id: Path<String>,
    principal_json: Json<Value>,
    schemas_repository: Data<Arc<SchemaRepository>>,
    principals_repository: Data<Arc<PrincipalRepository>>,
) -> Result<impl Responder> {
    let schema: Schema = schemas_repository
        .get(schema_id.clone())
        .await?
        .try_into()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let entity = Entity::from_json_value(principal_json.into_inner(), Some(&schema))
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let uid = entity.uid().to_string();
    let principal_identity = PrincipalIdentity::new(schema_id.clone(), entity.uid());
    principals_repository.upsert(principal_identity, entity).await?;
    let response = PrincipalCreateResponse { uid: uid.to_string() };
    Ok(Json(response))
}

#[utoipa::path(context_path = "/principal/",
    responses(
        (status = OK, body = Value),
        (status = NOT_FOUND, description = "Principal does not exist")
    ),
    security(
        ("internal" = [])
    )
)]
#[get("{schema}/{id}")]
async fn get_principal(path: Path<(String, String)>, data: Data<Arc<PrincipalRepository>>) -> Result<impl Responder> {
    let (schema, id) = path.into_inner();
    let id = EntityUid::from_str(&id).map_err(actix_web::error::ErrorBadRequest)?;
    let principal_identity = PrincipalIdentity::new(schema, id);
    let principal = data.get(principal_identity).await?;
    let json = principal
        .to_json_value()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(Json(json))
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/principal").service(post_principal).service(get_principal)
}
