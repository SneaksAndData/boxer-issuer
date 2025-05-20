use super::test_data::{external_identity, principal_type, user_name};
use async_trait::async_trait;
use boxer_issuer::services::base::upsert_repository::PrincipalAssociationRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn new() -> Arc<PrincipalAssociationRepository> {
     Arc::new(RwLock::new(HashMap::new()))
}

#[async_trait]
pub trait PrincipalAssociationRepositoryExt {
    async fn with_default_data(self) -> Arc<PrincipalAssociationRepository>;
}

#[async_trait]
impl PrincipalAssociationRepositoryExt for Arc<PrincipalAssociationRepository> {
    async fn with_default_data(self) -> Arc<PrincipalAssociationRepository> {
        self.upsert(external_identity(), (principal_type(), user_name())).await.unwrap();
        self
    }
}
