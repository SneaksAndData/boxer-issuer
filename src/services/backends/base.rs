use crate::services::backends::in_memory::InMemoryBackend;
use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::base::upsert_repository::IdentityRepository;
use crate::services::base::upsert_repository::PrincipalRepository;
use crate::services::base::upsert_repository::{PrincipalAssociationRepository, SchemaRepository};
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
/// A trait for managing application configuration updates.
pub trait BackendConfigurationManager {
    /// Returns the type of backend used by the application.
    async fn configure(&self, backend: Arc<dyn Backend>) -> Result<Arc<dyn Backend>>;
}

pub async fn load_backend(backend_type: BackendType, cm: &dyn BackendConfigurationManager) -> Result<Arc<dyn Backend>> {
    let backend: Arc<dyn Backend> = match backend_type {
        BackendType::InMemory => Arc::new(InMemoryBackend::new()),
        BackendType::Kubernetes => Arc::new(KubernetesBackend::new()),
    };
    Ok(cm.configure(backend).await?)
}
