use actix_web::dev::HttpServiceFactory;
use actix_web::get;
use actix_web::web;

#[utoipa::path(
    context_path = "/health",
    responses((status = OK, body = String)),
    responses((status = StatusCode::INTERNAL_SERVER_ERROR, description = "Service bad")),
)]
#[get("")]
pub async fn get_health() -> actix_web::Result<String> {
    Ok("OkK".into())
}

#[utoipa::path(
    context_path = "/health",
    responses((status = OK, body = String)),
    responses((status = StatusCode::INTERNAL_SERVER_ERROR, description = "Service not ready")),
)]
#[get("/probe")]
pub async fn get_health_probe() -> actix_web::Result<String> {
    Ok("OK".into())
}

pub fn urls() -> impl HttpServiceFactory {
    web::scope("/health").service(get_health_probe).service(get_health)
}
