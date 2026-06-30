#![cfg(test)]

mod fixtures;

use crate::fixtures::TestServerHandles;
use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use boxer_core::services::audit::AuditService;
use boxer_issuer_http::services::principal_service::PrincipalServiceTrait;
use boxer_issuer_http::services::token_service::TokenProvider;
use fixtures::{with_logging, with_test_server};
use mockall::mock;
use reqwest::Client;
use rstest::rstest;
use std::time::Duration;

#[rstest]
#[timeout(Duration::from_secs(15))]
#[actix_web::test]
async fn it_works(_with_logging: (), #[future] with_test_server: TestServerHandles) {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let (server_handle, thread_handle) = with_test_server.await;
    let client = Client::new();
    let external_token = get_external_token(&client).await.expect("Failed to get external token");
    println!("EXTERNAL_TOKEN: {}", external_token);
    let internal_token = get_internal_token(&client, external_token)
        .await
        .expect("Failed to get internal token");
    println!("INTERNAL_TOKEN: {}", internal_token);

    let validation_result = client
        .get("http://localhost:5555/validator/api/v1/token/review")
        // .get("http://localhost:8081/api/v1/token/review")
        .header("X-Original-Url", "http://example.com/api/v1/example/")
        .header("X-Original-Method", "GET")
        .bearer_auth(internal_token)
        .send()
        .await
        .expect("Failed to call token review endpoint");

    println!("Validation result: {:?}", validation_result);

    assert_eq!(validation_result.status(), 200);
    server_handle.stop(true).await;
    thread_handle.await.unwrap().expect("Failed to join server thread");
}

async fn get_external_token(client: &Client) -> Result<String> {
    let response = client
        .post("http://localhost:5555/auth/realms/master/protocol/openid-connect/token")
        .form(&[
            ("client_id", "test_client"),
            ("client_secret", "test_client_secret"),
            ("username", "test_root"),
            ("password", "test-root-password"),
            ("grant_type", "password"),
        ])
        .send()
        .await?;

    let body = response.text().await?;
    let claims = serde_json::from_str::<serde_json::Value>(&body)?;

    let access_token = claims["access_token"]
        .as_str()
        .ok_or(anyhow::anyhow!("access_token not found in response"))?;
    Ok(access_token.to_string())
}

async fn get_internal_token(client: &Client, external_token: String) -> Result<String> {
    Ok(client
        .get("http://localhost:8080/api/v1/token/keycloak")
        .bearer_auth(external_token)
        .send()
        .await?
        .text()
        .await?)
}

mock! {
    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }

}
