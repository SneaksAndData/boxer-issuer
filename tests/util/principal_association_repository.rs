use boxer_issuer::services::base::upsert_repository::PrincipalAssociationRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn new() -> Arc<PrincipalAssociationRepository> {
    Arc::new(RwLock::new(HashMap::new()))
}
