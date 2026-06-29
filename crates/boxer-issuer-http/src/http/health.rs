use actix_web::dev::HttpServiceFactory;
use actix_web::get;
use actix_web::web;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
mod tests;

#[utoipa::path(
    context_path = "/health",
    responses((status = OK, body = String)),
    responses((status = StatusCode::INTERNAL_SERVER_ERROR, description = "Service bad")),
)]
#[get("")]
pub async fn get_health() -> actix_web::Result<String> {
    Ok("ok".into())
}

#[utoipa::path(
    context_path = "/health",
    responses((status = OK, body = String)),
    responses((status = StatusCode::SERVICE_UNAVAILABLE, description = "Service not ready")),
)]
#[get("/ready")]
pub async fn get_health_probe(readiness_state: web::Data<Arc<AtomicBool>>) -> actix_web::Result<String> {
    if readiness_state.load(Ordering::Acquire) {
        return Ok("ok".into());
    }
    Err(actix_web::error::ErrorServiceUnavailable("Service not ready"))
}

pub fn urls() -> impl HttpServiceFactory {
    web::scope("/health").service(get_health_probe).service(get_health)
}
