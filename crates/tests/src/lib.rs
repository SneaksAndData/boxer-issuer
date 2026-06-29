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
use mockall::mock;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[actix_web::test]
async fn it_works() {
    let server = build_server().await;
    let thread = tokio::spawn(server);

    thread.abort();
    assert_eq!(true, false);
}

async fn build_server() -> Server {
    let app_settings = AppSettings {
        deploy_environment: "integration-tests".to_string(),
        instance_name: "test".to_string(),
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
                resource_owner_label: "integration-tests".to_string(),
            }),
        },
        token_settings: TokenSettings {
            issuer: "integration-tests".to_string(),
            audience: "integration-tests".to_string(),
            key_id: "key-id".to_string(),
            key: "key".to_string(),
            content_encryption: "encryption".to_string(),
        },
    };

    let current_backend = load_backend(BackendType::Kubernetes, &app_settings)
        .await
        .expect("Failed to load backend");

    let principal_service = Arc::new(MockPrincipalService::new());
    let token_provider = Arc::new(MockTokenProvider::new());
    let audit_service = Arc::new(MockAuditService::new());
    let audit_writer = Arc::new(MockAuditWriter::new());
    let readiness_state = Arc::new(AtomicBool::new(true));
    boxer_issuer_http::start_api_server(
        current_backend,
        token_provider,
        audit_service,
        audit_writer,
        readiness_state,
        principal_service,
        app_settings,
    )
    .expect("Start api server failed")
}

mock! {

    pub TokenProvider {}

    #[async_trait]
    impl TokenProvider for TokenProvider {
        async fn issue_token( &self, external_identity_provider: ExternalIdentityProvider, external_token: boxer_core::models::external_token::ExternalToken, ) -> Result<String>;
    }
}

mock! {
    pub AuditService {}

    impl AuditService for AuditService {
        fn record_authorization(&self, event: AuthorizationAuditEvent) -> Result<()>;
        fn record_resource_deletion(&self, event: ResourceDeleteAuditEvent) -> Result<()>;
        fn record_resource_modification(&self, event: ResourceModificationAuditEvent) -> Result<()>;
        fn record_token_validation(&self, event: TokenValidationEvent) -> Result<()>;
    }

}

mock! {
    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }

}

mock! {
    pub PrincipalService {}

    #[async_trait]
    impl PrincipalServiceTrait for PrincipalService {
        async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal>;
        async fn get_validator_schema(&self, external_identity: ExternalIdentity) -> Result<String>;
        async fn get_schemas(&self, schema_id: String) -> Result<SchemaFragment, anyhow::Error>;
    }
}
