mod principal_repository;
mod identity_repository;
mod principal_association_repository;
mod schema_repository;
mod common;

use crate::services::backends::base::Backend;
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository, SchemaRepository,
};
use std::sync::Arc;

pub struct KubernetesBackend {
    pub schemas_repository: Option<Arc<SchemaRepository>>,
    pub entities_repository: Option<Arc<PrincipalRepository>>,
    pub principal_association_repository: Option<Arc<PrincipalAssociationRepository>>,
    pub identity_repository: Option<Arc<IdentityRepository>>,
}

impl KubernetesBackend {
    pub fn new() -> Self {
        KubernetesBackend {
            schemas_repository: None,
            entities_repository: None,
            principal_association_repository: None,
            identity_repository: None,
        }
    }
}

impl Backend for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schemas_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_entities_repository(&self) -> Arc<PrincipalRepository> {
        self.entities_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository> {
        self.principal_association_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_identity_repository(&self) -> Arc<IdentityRepository> {
        self.identity_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}
