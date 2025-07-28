use crate::http::errors::*;
use crate::models::principal::Principal;
use crate::services::base::upsert_repository::{PrincipalIdentity, PrincipalRepository};
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Path};
use actix_web::{get, post, web, HttpResponse};
use boxer_core::services::base::types::SchemaRepository;
use cedar_policy::{Entity, Schema};
use std::sync::Arc;

#[utoipa::path(context_path = "/principal/", responses((status = OK)))]
#[post("{schema}")]
async fn post_principal(
    schema_id: Path<String>,
    principal_json: String,
    schemas_repository: Data<Arc<SchemaRepository>>,
    principals_repository: Data<Arc<PrincipalRepository>>,
) -> Result<HttpResponse> {
    let schema_fragment = schemas_repository.get(schema_id.clone()).await?;
    let schema: Schema = schema_fragment.try_into()?;
    let entity = Entity::from_json_str(&principal_json, Some(&schema))?;
    principals_repository
        .upsert(
            PrincipalIdentity::from((schema_id.clone(), entity.uid())),
            Principal::new(entity, schema_id.into_inner()),
        )
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/principal/", responses((status = OK)))]
#[get("{schema}/{id}")]
async fn get_principal(path: Path<(String, String)>, data: Data<Arc<PrincipalRepository>>) -> Result<String> {
    let (schema, id) = path.into_inner();
    let principal = data.get(PrincipalIdentity::from((schema, id))).await?;
    let json = principal.get_entity().to_json_string()?;
    Ok(json)
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/principal").service(post_principal).service(get_principal)
}
