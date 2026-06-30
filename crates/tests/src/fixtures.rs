use crate::MockAuditWriter;
use actix_web::dev::ServerHandle;
use boxer_core::services::observability::open_telemetry::logging::settings::LogSettings;
use boxer_core::services::observability::open_telemetry::metrics::settings::MetricsSettings;
use boxer_core::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use boxer_core::services::observability::open_telemetry::tracing::settings::TracingSettings;
use boxer_issuer_http::services::backends::base::{load_backend, BackendType};
use boxer_issuer_http::services::configuration::models::{
    AppSettings, BackendSettings, InitializationSettings, KubernetesBackendSettings, TokenSettings,
};
use rstest::fixture;
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

pub type TestServerHandles = (ServerHandle, JoinHandle<std::io::Result<()>>);
#[fixture]
pub async fn with_test_server() -> TestServerHandles {
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
            key: "KZW3kJpPUQse99az".to_string(),
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
    (handle, thread)
}
