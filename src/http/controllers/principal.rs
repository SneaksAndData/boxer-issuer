use crate::http::errors::*;
use crate::models::principal::Principal;
use crate::services::base::upsert_repository::PrincipalsRepository;
use crate::services::base::upsert_repository::SchemaRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{BytesMut, Data, Path, Payload};
use actix_web::{get, post, web, HttpResponse};
use cedar_policy::{Entity, Schema};
use futures::StreamExt;
use std::sync::Arc;

const MAX_PRINCIPAL_SIZE: usize = 262_144; // max payload size is 256k

#[utoipa::path(context_path = "/principal/", responses((status = OK)))]
#[post("{schema}/{type}/{id}")]
async fn post(
    path: Path<(String, String, String)>,
    mut payload: Payload,
    schemas_repository: Data<Arc<SchemaRepository>>,
    principals_repository: Data<Arc<PrincipalsRepository>>,
) -> Result<HttpResponse> {
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_PRINCIPAL_SIZE {
            return Err(Error::new("Submitted principal exceeds max size of 256k"));
        }
        body.extend_from_slice(&chunk);
    }
    let principal_json = String::from_utf8_lossy(&body);
    let (schema_id, type_, id) = path.into_inner();
    let schema_fragment = schemas_repository.get(schema_id.clone()).await?;
    let schema: Schema = schema_fragment.try_into()?;
    let entity = Entity::from_json_str(&principal_json, Some(&schema))?;
    principals_repository
        .upsert((type_, id), Principal::new(entity, schema_id))
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/principal/", responses((status = OK)))]
#[get("{schema}/{type}/{id}")]
async fn get(path: Path<(String, String, String)>, data: Data<Arc<PrincipalsRepository>>) -> Result<String> {
    let (_, type_, id) = path.into_inner();
    let principal = data.get((type_, id)).await?;
    let json = principal.get_entity().to_json_string()?;
    Ok(json)
}

#[utoipa::path(context_path = "/principal/", responses((status = OK)))]
#[get("{schema}/{type}/{id}")]
async fn delete(path: Path<(String, String, String)>, data: Data<Arc<PrincipalsRepository>>) -> Result<HttpResponse> {
    let (_, type_, id) = path.into_inner();
    data.delete((type_, id)).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/principal").service(post).service(get).service(delete)
}
