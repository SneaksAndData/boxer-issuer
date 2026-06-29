use actix_web::web;
use actix_web::{App, http::StatusCode, test};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[actix_web::test]
async fn test_health_returns_ok() {
    // The purpose of this test is API documentation, not so much covering the business logic
    let readiness_state = web::Data::new(Arc::new(AtomicBool::new(true)));
    let app = test::init_service(App::new().app_data(readiness_state).service(super::urls())).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_health_probe_returns_service_unavailable_when_not_ready() {
    let readiness_state = web::Data::new(Arc::new(AtomicBool::new(false)));
    let app = test::init_service(App::new().app_data(readiness_state).service(super::urls())).await;

    let req = test::TestRequest::get().uri("/health/ready").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[actix_web::test]
async fn test_health_probe_returns_ok_when_ready() {
    let readiness_state = web::Data::new(Arc::new(AtomicBool::new(true)));
    let app = test::init_service(App::new().app_data(readiness_state).service(super::urls())).await;
    let req = test::TestRequest::get().uri("/health/ready").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}
