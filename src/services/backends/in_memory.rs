use crate::services::backends::base::{
    EntitiesRepositorySource, ExternalIdentityValidatorProviderSource, IdentityProviderRepositorySource,
    IdentityRepositorySource, IssuerBackend, PrincipalAssociationRepositorySource,
};
use crate::services::base::upsert_repository::{
    IdentityProviderRepository, IdentityRepository, PrincipalAssociationRepository, PrincipalRepository,
};
use crate::services::configuration::models::BackendSettings;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use async_trait::async_trait;
use boxer_core::services::backends::{Backend, BackendConfiguration, SchemaRepositorySource};
use boxer_core::services::base::types::SchemaRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryBackend {
    pub schemas_repository: Arc<SchemaRepository>,
    pub entities_repository: Arc<PrincipalRepository>,
    pub principal_association_repository: Arc<PrincipalAssociationRepository>,
    pub identity_repository: Arc<IdentityRepository>,
    pub identity_provider_repository: Arc<IdentityProviderRepository>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        let schemas_repository = Arc::new(RwLock::new(HashMap::new()));
        let entities_repository = Arc::new(RwLock::new(HashMap::new()));
        let principal_association_repository = Arc::new(RwLock::new(HashMap::new()));
        let identity_repository = Arc::new(RwLock::new(HashMap::new()));
        let identity_provider_repository = Arc::new(RwLock::new(HashMap::new()));
        InMemoryBackend {
            schemas_repository,
            entities_repository,
            principal_association_repository,
            identity_repository,
            identity_provider_repository,
        }
    }
}

impl SchemaRepositorySource for InMemoryBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        Arc::clone(&self.schemas_repository)
    }
}

impl EntitiesRepositorySource for InMemoryBackend {
    fn get_entities_repository(&self) -> Arc<PrincipalRepository> {
        Arc::clone(&self.entities_repository)
    }
}

impl PrincipalAssociationRepositorySource for InMemoryBackend {
    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository> {
        Arc::clone(&self.principal_association_repository)
    }
}

impl IdentityRepositorySource for InMemoryBackend {
    fn get_identity_repository(&self) -> Arc<IdentityRepository> {
        Arc::clone(&self.identity_repository)
    }
}

impl IdentityProviderRepositorySource for InMemoryBackend {
    fn get_identity_provider_repository(&self) -> Arc<IdentityProviderRepository> {
        Arc::clone(&self.identity_provider_repository)
    }
}

impl ExternalIdentityValidatorProviderSource for InMemoryBackend {
    fn get_external_identity_validator_provider(&self) -> Arc<dyn ExternalIdentityValidatorProvider> {
        todo!()
    }
}

impl Backend for InMemoryBackend {
    // Nothing here, as this is a marker trait
}

impl IssuerBackend for InMemoryBackend {
    // Nothing here, as this is a marker trait
}

#[async_trait]
impl BackendConfiguration for InMemoryBackend {
    type BackendSettings = BackendSettings;
    type InitializedBackend = InMemoryBackend;

    async fn configure(mut self, _: &BackendSettings, _: String) -> anyhow::Result<Arc<Self::InitializedBackend>> {
        // No additional configuration needed for InMemoryBackend
        Ok(Arc::new(self))
    }
}
