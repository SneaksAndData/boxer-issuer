#![cfg(test)]

mod fixtures;

use crate::fixtures::{TestServerHandles, external_token, token_review_endpoint};
use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use boxer_core::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use boxer_core::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use boxer_core::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
use fixtures::{with_logging, with_test_server};
use mockall::mock;
use reqwest::Client;
use rstest::rstest;
use std::net::SocketAddr;
use std::time::Duration;

#[rstest]
#[timeout(Duration::from_secs(15))]
#[actix_web::test]
async fn test_internal_token_issuance(
    _with_logging: (),
    #[future] with_test_server: TestServerHandles,
    token_review_endpoint: String,
    #[future] external_token: String,
) -> () {
    // Arrange
    let (server_handle, thread_handle, server_address) = with_test_server.await;
    let external_token = external_token.await;
    let internal_token = get_internal_token(external_token, server_address)
        .await
        .expect("Failed to get internal token");

    // Act
    let validation_result = Client::new()
        .get(token_review_endpoint)
        .header("X-Original-Url", "http://example.com/api/v1/example/")
        .header("X-Original-Method", "GET")
        .bearer_auth(internal_token)
        .send()
        .await
        .expect("Failed to call token review endpoint");

    // Assert
    assert_eq!(validation_result.status(), 200);

    // Cleanup
    server_handle.stop(true).await;
    thread_handle.await.unwrap().expect("Failed to join server thread");
}

#[rstest]
#[timeout(Duration::from_secs(15))]
#[actix_web::test]
async fn test_identity_provider_does_not_exist(_with_logging: (), #[future] external_token: String) -> () {
    // Arrange
    let mut audit_writer = MockAuditWriter::new();
    audit_writer
        .expect_write()
        .withf(|event| {
            let error_text = "Error response with status: 401 Unauthorized: Unauthorized, err: \
            Resource not found: Resource of kind 'IdentityProviderDocument' with name: 'non-exitent-idp',  \
            namespace 'default' not found";

            matches!(
                event,
                AuditEvent::Final(FinalAuditEvent {
                    policy_evaluation_result: PolicyEvaluationResult {
                        decision: Decision::Deny,
                        reason: Some(Reason {
                            errors,
                            ..
                        }),
                        ..
                    },
                    ..
                }) if errors.contains(error_text)
            )
        })
        .times(1)
        .returning(|_event| ());

    let (server_handle, thread_handle, server_address) = with_test_server(audit_writer).await;
    let external_token = external_token.await;

    // Act
    let result = Client::new()
        .get(format!("http://{}/api/v1/token/non-exitent-idp", server_address))
        .bearer_auth(external_token)
        .send()
        .await;

    // Assert
    assert_eq!(result.unwrap().status(), 401);

    // Cleanup
    server_handle.stop(true).await;
    thread_handle.await.unwrap().expect("Failed to join server thread");
}

async fn get_internal_token(external_token: String, server_address: SocketAddr) -> Result<String> {
    Ok(Client::new()
        .get(format!("http://{}/api/v1/token/keycloak", server_address))
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
