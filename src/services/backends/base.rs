use crate::services::backends::in_memory::InMemoryBackend;
use crate::services::base::upsert_repository::IdentityRepository;
use crate::services::base::upsert_repository::PrincipalRepository;
use crate::services::base::upsert_repository::{PrincipalAssociationRepository, SchemaRepository};
use crate::services::configuration_manager::ConfigurationManager;
use std::sync::Arc;

#[allow(dead_code)]
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

pub fn load_backend(cm: &dyn ConfigurationManager) -> impl Backend {
    match cm.get_backend_type() {
        BackendType::InMemory => InMemoryBackend::new(),
        BackendType::Kubernetes => {
            // Implement Kubernetes backend creation logic here
            unimplemented!()
        }
    }
}
