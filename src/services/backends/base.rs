use crate::services::backends::in_memory::InMemoryBackend;
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

pub trait Backend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository>;
    fn get_entities_repository(&self) -> Arc<PrincipalRepository>;
    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository>;
    fn get_identity_repository(&self) -> Arc<IdentityRepository>;
}

#[async_trait]
/// A trait for managing application configuration updates.
pub trait BackendConfigurationManager {
    /// Returns the type of backend used by the application.
    async fn configure(&self, backend: &mut dyn Backend) -> Result<()>;
}

pub async fn load_backend(backend_type: BackendType, cm: &dyn BackendConfigurationManager) -> Result<impl Backend> {
    let mut backend = match backend_type {
        BackendType::InMemory => InMemoryBackend::new(),
        BackendType::Kubernetes => {
            // Implement Kubernetes backend creation logic here
            unimplemented!()
        }
    };
    cm.configure(&mut backend).await?;
    Ok(backend)
}
