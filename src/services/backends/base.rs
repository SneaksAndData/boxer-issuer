use crate::services::backends::in_memory::InMemoryBackend;
use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::base::upsert_repository::PrincipalAssociationRepository;
use crate::services::base::upsert_repository::PrincipalRepository;
use crate::services::base::upsert_repository::{IdentityProviderRepository, IdentityRepository};
use crate::services::configuration::models::AppSettings;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use anyhow::Result;
use boxer_core::services::backends::{Backend, BackendConfiguration};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub enum BackendType {
    InMemory,
    Kubernetes,
}

pub trait EntitiesRepositorySource {
    #[allow(dead_code)]
    fn get_entities_repository(&self) -> Arc<PrincipalRepository>;
}

pub trait PrincipalAssociationRepositorySource {
    #[allow(dead_code)]
    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository>;
}

pub trait IdentityRepositorySource {
    #[allow(dead_code)]
    fn get_identity_repository(&self) -> Arc<IdentityRepository>;
}

pub trait IdentityProviderRepositorySource {
    #[allow(dead_code)]
    fn get_identity_provider_repository(&self) -> Arc<IdentityProviderRepository>;
}

pub trait ExternalIdentityValidatorProviderSource {
    #[allow(dead_code)]
    fn get_external_identity_validator_provider(&self) -> Arc<dyn ExternalIdentityValidatorProvider>;
}

pub trait IssuerBackend:
    Send
    + Sync
    + Backend
    + EntitiesRepositorySource
    + PrincipalAssociationRepositorySource
    + IdentityRepositorySource
    + IdentityProviderRepositorySource
    + ExternalIdentityValidatorProviderSource
{
}

pub async fn load_backend(backend_type: BackendType, cm: &AppSettings) -> Result<Arc<dyn IssuerBackend>> {
    let backend: Arc<dyn IssuerBackend> = match backend_type {
        BackendType::InMemory => {
            InMemoryBackend::new()
                .configure(&cm.backend, cm.instance_name.clone())
                .await?
        }
        BackendType::Kubernetes => {
            KubernetesBackend::new()
                .configure(&cm.backend, cm.instance_name.clone())
                .await?
        }
    };
    Ok(backend)
}
