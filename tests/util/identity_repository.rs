use super::test_data::{external_identity, external_identity_raw};
use async_trait::async_trait;
use boxer_issuer::services::base::upsert_repository::IdentityRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn new() -> Arc<IdentityRepository> {
    Arc::new(RwLock::new(HashMap::new()))
}

#[async_trait]
pub trait IdentityRepositoryExt {
    async fn with_default_data(self) -> Arc<IdentityRepository>;
}

#[async_trait]
impl IdentityRepositoryExt for Arc<IdentityRepository> {
    async fn with_default_data(self) -> Arc<IdentityRepository> {
        self.upsert(external_identity_raw(), external_identity()).await.unwrap();
        self
    }
}
