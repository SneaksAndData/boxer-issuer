use crate::MockAuditWriter;
use actix_web::dev::ServerHandle;
use anyhow::{Result, anyhow};
use boxer_core::services::observability::open_telemetry::logging::settings::LogSettings;
use boxer_core::services::observability::open_telemetry::metrics::settings::MetricsSettings;
use boxer_core::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use boxer_core::services::observability::open_telemetry::tracing::settings::TracingSettings;
use boxer_core::testing::get_kubeconfig;
use boxer_issuer_http::services::backends::base::{BackendType, load_backend};
use boxer_issuer_http::services::configuration::models::{
    AppSettings, BackendSettings, InitializationSettings, KubernetesBackendSettings, TokenSettings,
};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};
use rstest::fixture;
use serde_json::{Value, from_str};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[fixture]
pub fn with_logging() -> () {
    let _ = env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

#[fixture]
pub fn token_review_endpoint() -> String {
    "http://localhost:5555/validator/api/v1/token/review".to_string()
}

#[fixture]
pub async fn external_token() -> String {
    const KEYCLOAK_URL: &str = "http://localhost:5555/auth/realms/master/protocol/openid-connect/token";
    let client = reqwest::Client::new();
    let response = client
        .post(KEYCLOAK_URL)
        .form(&[
            ("client_id", "test_client"),
            ("client_secret", "test_client_secret"),
            ("username", "test_root"),
            ("password", "test-root-password"),
            ("grant_type", "password"),
        ])
        .send()
        .await
        .expect("Failed to send request to Keycloak");

    let body = response
        .text()
        .await
        .expect("Failed to read response body from Keycloak");
    let claims = from_str::<Value>(&body).expect("Failed to parse response body from Keycloak as JSON");

    let access_token = claims["access_token"]
        .as_str()
        .expect("Failed to extract access_token from Keycloak response");
    access_token.to_string()
}

pub type TestServerHandles = (ServerHandle, JoinHandle<std::io::Result<()>>, SocketAddr);
#[fixture]
pub async fn with_test_server() -> TestServerHandles {
    let server_address = "127.0.0.1:8080".parse().unwrap();
    let app_settings = AppSettings {
        deploy_environment: "integration-tests".to_string(),
        instance_name: "integration-tests".to_string(),
        listen_address: SocketAddr::from(server_address),
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
            issuer: "boxer.sneaksanddata.com".to_string(),
            audience: "boxer.sneaksanddata.com".to_string(),
            key_id: "default".to_string(),
            key: get_singing_key().await.expect("Failed to get signing key"),
            content_encryption: "A128CBC-HS256".to_string(),
        },
    };

    let current_backend = load_backend(BackendType::Kubernetes, &app_settings)
        .await
        .expect("Failed to load backend");

    let mut audit_writer = MockAuditWriter::new();
    audit_writer.expect_write().returning(|_event| ());

    let audit_writer = Arc::new(audit_writer);
    let server = boxer_issuer_http::start_api_server(current_backend, audit_writer, app_settings, "test")
        .expect("Start api server failed");

    let handle = server.handle();
    let thread = tokio::spawn(server);
    (handle, thread, server_address)
}

async fn get_singing_key() -> Result<String> {
    let kubeconfig = get_kubeconfig().await?;
    let client = Client::try_from(kubeconfig)?;
    let secrets: Api<Secret> = Api::namespaced(client, "default");
    let secret = secrets.get("boxer-issuer-token-settings").await?;
    let key = secret
        .data
        .as_ref()
        .and_then(|data| data.get("BOXER__TOKEN_SETTINGS__KEY"))
        .ok_or_else(|| anyhow!("missing BOXER__TOKEN_SETTINGS__KEY in boxer-issuer-token-settings"))?;

    Ok(String::from_utf8(key.0.clone())?)
}
