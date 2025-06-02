use crate::services::backends::base::Backend;
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository, SchemaRepository,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryBackend {
    pub schemas_repository: Arc<SchemaRepository>,
    pub entities_repository: Arc<PrincipalRepository>,
    pub principal_association_repository: Arc<PrincipalAssociationRepository>,
    pub identity_repository: Arc<IdentityRepository>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        let schemas_repository = Arc::new(RwLock::new(HashMap::new()));
        let entities_repository = Arc::new(RwLock::new(HashMap::new()));
        let principal_association_repository = Arc::new(RwLock::new(HashMap::new()));
        let identity_repository = Arc::new(RwLock::new(HashMap::new()));
        InMemoryBackend {
            schemas_repository,
            entities_repository,
            principal_association_repository,
            identity_repository,
        }
    }
}

impl Backend for InMemoryBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        Arc::clone(&self.schemas_repository)
    }

    fn get_entities_repository(&self) -> Arc<PrincipalRepository> {
        Arc::clone(&self.entities_repository)
    }

    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository> {
        Arc::clone(&self.principal_association_repository)
    }

    fn get_identity_repository(&self) -> Arc<IdentityRepository> {
        Arc::clone(&self.identity_repository)
    }
}
