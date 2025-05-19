use crate::http::errors::*;
use crate::services::base::upsert_repository::SchemaRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{BytesMut, Data, Path, Payload};
use actix_web::{delete, get, post, web, HttpResponse};
use cedar_policy::SchemaFragment;
use futures::StreamExt;
use std::sync::Arc;

const MAX_SCHEMA_SIZE: usize = 262_144; // max payload size is 256k

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[post("{id}")]
async fn post(id: Path<String>, mut payload: Payload, data: Data<Arc<SchemaRepository>>) -> Result<HttpResponse> {
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SCHEMA_SIZE {
            return Err(Error::new("Submitted schema exceeds max size of 256k"));
        }
        body.extend_from_slice(&chunk);
    }
    let schema_json = String::from_utf8_lossy(&body);
    let schema = SchemaFragment::from_json_str(&schema_json)?;
    data.upsert(id.to_string(), schema).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[get("{id}")]
async fn get(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<String> {
    let schema = data.get(id.to_string()).await?;
    let result = schema.to_json_string()?;
    Ok(result)
}

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[delete("{id}")]
async fn delete(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<HttpResponse> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/schema").service(post).service(get).service(delete)
}
