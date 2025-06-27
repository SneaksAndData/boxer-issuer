use crate::services::backends::in_memory::InMemoryBackend;
use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::base::upsert_repository::IdentityRepository;
use crate::services::base::upsert_repository::PrincipalRepository;
use crate::services::base::upsert_repository::{PrincipalAssociationRepository, SchemaRepository};
use crate::services::configuration::models::{AppSettings, BackendSettings};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub enum BackendType {
    InMemory,
    Kubernetes,
}

pub trait Backend: Send + Sync {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository>;
    fn get_entities_repository(&self) -> Arc<PrincipalRepository>;
    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository>;
    fn get_identity_repository(&self) -> Arc<IdentityRepository>;
}

#[async_trait]
pub trait BackendConfiguration: Send + Sync + Sized {
    async fn configure(mut self, cm: &BackendSettings) -> Result<Self>;
}

pub async fn load_backend(backend_type: BackendType, cm: &AppSettings) -> Result<Arc<dyn Backend>> {
    let backend: Arc<dyn Backend> = match backend_type {
        BackendType::InMemory => Arc::new(InMemoryBackend::new().configure(&cm.backend).await?),
        BackendType::Kubernetes => Arc::new(KubernetesBackend::new().configure(&cm.backend).await?),
    };
    Ok(backend)
}
