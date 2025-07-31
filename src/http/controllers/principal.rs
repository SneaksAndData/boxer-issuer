use crate::http::errors::*;
use crate::models::principal::Principal;
use crate::services::base::upsert_repository::{PrincipalIdentity, PrincipalRepository};
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, web, Responder};
use boxer_core::services::base::types::SchemaRepository;
use cedar_policy::{Entity, Schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct PrincipalCreateResponse {
    uid: String,
}

#[utoipa::path(context_path = "/principal/", request_body = Value, responses((status = OK, body = PrincipalCreateResponse)))]
#[post("{schema}")]
async fn post_principal(
    schema_id: Path<String>,
    principal_json: Json<Value>,
    schemas_repository: Data<Arc<SchemaRepository>>,
    principals_repository: Data<Arc<PrincipalRepository>>,
) -> Result<impl Responder> {
    let schema_fragment = schemas_repository.get(schema_id.clone()).await?;
    let schema: Schema = schema_fragment.try_into()?;
    let entity = Entity::from_json_value(principal_json.into_inner(), Some(&schema))?;
    let uid = entity.uid().to_string();
    principals_repository
        .upsert(
            PrincipalIdentity::from((schema_id.clone(), entity.uid())),
            Principal::new(entity, schema_id.into_inner()),
        )
        .await?;
    let response = PrincipalCreateResponse { uid: uid.to_string() };
    Ok(Json(response))
}

#[utoipa::path(context_path = "/principal/", responses((status = OK, body = Value)))]
#[get("{schema}/{id}")]
async fn get_principal(path: Path<(String, String)>, data: Data<Arc<PrincipalRepository>>) -> Result<impl Responder> {
    let (schema, id) = path.into_inner();
    let principal = data.get(PrincipalIdentity::from((schema, id))).await?;
    let json = principal.get_entity().to_json_value()?;
    Ok(Json(json))
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/principal").service(post_principal).service(get_principal)
}
