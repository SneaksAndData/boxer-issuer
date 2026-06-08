use actix_web::dev::HttpServiceFactory;
use actix_web::get;
use actix_web::web;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    responses((status = StatusCode::SERVICE_UNAVAILABLE, description = "Service not ready")),
)]
#[get("/probe")]
pub async fn get_health_probe(readiness_state: web::Data<Arc<AtomicBool>>) -> actix_web::Result<String> {
    if readiness_state.load(Ordering::Acquire) {
        return Ok("OK".into());
    }
    Err(actix_web::error::ErrorServiceUnavailable("Service not ready"))
}

pub fn urls() -> impl HttpServiceFactory {
    web::scope("/health").service(get_health_probe).service(get_health)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_health_probe_returns_service_unavailable_when_not_ready() {
        let readiness_state = web::Data::new(Arc::new(AtomicBool::new(false)));
        let app = test::init_service(App::new().app_data(readiness_state).service(super::urls())).await;

        let req = test::TestRequest::get().uri("/health/probe").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[actix_web::test]
    async fn test_health_probe_returns_ok_when_ready() {
        let readiness_state = web::Data::new(Arc::new(AtomicBool::new(true)));
        let app = test::init_service(App::new().app_data(readiness_state).service(super::urls())).await;

        let req = test::TestRequest::get().uri("/health/probe").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
