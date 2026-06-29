#![cfg(test)]

use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::models::external_token::ExternalToken;
use boxer_core::services::audit::AuditService;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use boxer_core::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use boxer_core::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use boxer_core::services::audit::events::resource_modification_audit_event::ResourceModificationAuditEvent;
use boxer_core::services::audit::events::token_validation_event::TokenValidationEvent;
use boxer_issuer_http::models::api::external::identity::ExternalIdentity;
use boxer_issuer_http::models::api::external::identity_provider::ExternalIdentityProvider;
use boxer_issuer_http::services::backends::base::{BackendType, load_backend};
use boxer_issuer_http::services::configuration::models::{AppSettings, BackendSettings, KubernetesBackendSettings};
use boxer_issuer_http::services::principal_service::PrincipalServiceTrait;
use boxer_issuer_http::services::principal_service::principal::Principal;
use boxer_issuer_http::services::token_service::TokenProvider;
use mockall::mock;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[actix_web::test]
async fn it_works() {
    let thread = tokio::spawn(async {
        let app_settings = AppSettings {
            deploy_environment: "unit-tests".to_string(),
            backend: BackendSettings {
                kubernetes: Some(KubernetesBackendSettings {
                    exec: Some("/opt/homebrew/bin/kind get kubeconfig".to_string()),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };
        let current_backend = load_backend(BackendType::Kubernetes, &app_settings).await?;
        let token_provider = Arc::new(MockTokenProvider::new());
        let audit_service = Arc::new(MockAuditService::new());
        let audit_writer = Arc::new(MockAuditWriter::new());
        let readiness_state = Arc::new(AtomicBool::new(true));
        let principal_service = Arc::new(MockPrincipalService::new());
        let server = boxer_issuer_http::start_api_server(
            current_backend,
            token_provider,
            audit_service,
            audit_writer,
            readiness_state,
            principal_service,
            app_settings,
        )
        .await;
    });

    thread
    // assert!(assertserver, Ok(()));
}

mock! {

    pub TokenProvider {}

    impl TokenProvider for MockTokenProvider {
        async fn issue_token( &self, external_identity_provider: ExternalIdentityProvider, external_token: ExternalToken, ) -> Result<String>;
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

    impl PrincipalServiceTrait for PrincipalService {
        fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal>;
        fn get_validator_schema(&self, external_identity: ExternalIdentity) -> Result<String>;
    }
}
