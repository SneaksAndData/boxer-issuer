use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::configuration::models::AppSettings;
use anyhow::Result;
use boxer_core::services::backends::{Backend, BackendConfiguration};
use boxer_core::services::service_provider::ServiceProvider;
use boxer_core::services::token_service::internal_token_service::external_identity_validator_provider::ExternalIdentityValidatorProvider;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub enum BackendType {
    Kubernetes,
}

pub trait IssuerBackend:
    Send
    + Sync
    + Backend
    + ServiceProvider<Arc<PrincipalRepository>>
    + ServiceProvider<Arc<IdentityRepository>>
    + ServiceProvider<Arc<IdentityProviderRepository>>
    + ServiceProvider<Arc<dyn ExternalIdentityValidatorProvider>>
{
    fn readiness_state(&self) -> Arc<AtomicBool>;
}

pub async fn load_backend(backend_type: BackendType, cm: &AppSettings) -> Result<Arc<dyn IssuerBackend>> {
    let backend: Arc<dyn IssuerBackend> = match backend_type {
        BackendType::Kubernetes => {
            KubernetesBackend::new()
                .configure(&cm.backend, cm.instance_name.clone())
                .await?
        }
    };
    Ok(backend)
}
