#![cfg(test)]

use actix_web::dev::Server;
use anyhow::Result;
use async_trait::async_trait;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use boxer_core::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use boxer_core::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use boxer_core::services::audit::events::resource_modification_audit_event::ResourceModificationAuditEvent;
use boxer_core::services::audit::events::token_validation_event::TokenValidationEvent;
use boxer_core::services::audit::AuditService;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry::logging::settings::LogSettings;
use boxer_core::services::observability::open_telemetry::metrics::settings::MetricsSettings;
use boxer_core::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use boxer_core::services::observability::open_telemetry::tracing::settings::TracingSettings;
use boxer_issuer_http::models::api::external::identity::ExternalIdentity;
use boxer_issuer_http::models::api::external::identity_provider::ExternalIdentityProvider;
use boxer_issuer_http::services::backends::base::{load_backend, BackendType};
use boxer_issuer_http::services::configuration::models::{
    AppSettings, BackendSettings, InitializationSettings, KubernetesBackendSettings, TokenSettings,
};
use boxer_issuer_http::services::principal_service::principal::Principal;
use boxer_issuer_http::services::principal_service::PrincipalServiceTrait;
use boxer_issuer_http::services::token_service::TokenProvider;
use cedar_policy::SchemaFragment;
use env_filter::Builder;
use log::info;
use mockall::mock;
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;

#[actix_web::test]
async fn it_works() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    init_logging();

    let server = build_server().await;
    let handle = server.handle();
    let thread = tokio::spawn(server);

    let client = Client::new();
    let external_token = get_external_token(&client).await.expect("Failed to get external token");

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let internal_token = client
        .get("http://localhost:8080/api/v1/token/keycloak")
        .bearer_auth(external_token)
        .send()
        .await
        .unwrap();
    info!("Internal token response: {:?}", internal_token);

    // let external_token = handle.stop(true).await;

    assert_eq!(true, false);
}

fn init_logging() {
    let _ = env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

async fn build_server() -> Server {
    let app_settings = AppSettings {
        deploy_environment: "integration-tests".to_string(),
        instance_name: "integration-tests".to_string(),
        listen_address: SocketAddr::from(([127, 0, 0, 1], 8080)),
        init: InitializationSettings {
            backend_type: BackendType::Kubernetes,
        },
        opentelemetry: OpenTelemetrySettings {
            log_settings: LogSettings { enabled: false },
            metrics_settings: MetricsSettings { enabled: false },
            tracing_settings: TracingSettings { enabled: false },
        },
        backend: BackendSettings {
            kubernetes: Some(KubernetesBackendSettings {
                kubeconfig: None,
                exec: Some("/opt/homebrew/bin/kind get kubeconfig".to_string()),
                in_cluster: false,
                namespace: "default".to_string(),
                operation_timeout: Default::default(),
                resource_owner_label: "application/boxer-issuer".to_string(),
            }),
        },
        token_settings: TokenSettings {
            issuer: "integration-tests".to_string(),
            audience: "integration-tests".to_string(),
            key_id: "key-id".to_string(),
            key: "214bed5c9ddb129a6563565faf8416fe".to_string(),
            content_encryption: "A128CBC-HS256".to_string(),
        },
    };

    let current_backend = load_backend(BackendType::Kubernetes, &app_settings)
        .await
        .expect("Failed to load backend");

    let mut audit_writer = MockAuditWriter::new();
    audit_writer.expect_write().returning(|_event| ());

    let audit_writer = Arc::new(audit_writer);
    boxer_issuer_http::start_api_server(current_backend, audit_writer, app_settings, "test")
        .expect("Start api server failed")
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

mock! {
    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }

}
