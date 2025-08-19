use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use cedar_policy::SchemaFragment;
use serde_json::Value;
use std::sync::Arc;

#[utoipa::path(context_path = "/schema/",
    request_body = Value,
    responses(
        (status = OK),
    )
)]
#[post("{id}")]
async fn post_schema(
    id: Path<String>,
    schema_json: Json<Value>,
    data: Data<Arc<SchemaRepository>>,
) -> Result<HttpResponse> {
    let schema = SchemaFragment::from_json_value(schema_json.into_inner())
        .map_err(actix_web::error::ErrorInternalServerError)?;
    data.upsert(id.to_string(), schema).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/schema/",
    responses(
        (status = OK, body = Value),
        (status = NOT_FOUND, description = "Schema does not exist")
    )
)]
#[get("{id}")]
async fn get_schema(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<impl Responder> {
    let schema = data.get(id.to_string()).await?;
    let result = schema
        .to_json_value()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(Json(result))
}

#[utoipa::path(context_path = "/schema/",
    responses(
        (status = OK)
    )
)]
#[delete("{id}")]
async fn delete_schema(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<HttpResponse> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/schema")
        .service(post_schema)
        .service(get_schema)
        .service(delete_schema)
}
