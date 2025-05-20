use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use cedar_policy::Entity;
use tokio::sync::RwLock;
use boxer_issuer::models::principal::Principal;
use boxer_issuer::services::base::upsert_repository::PrincipalsRepository;
use super::test_data::{principal_type, schema, schema_name, user_name, USER};

pub fn new() -> Arc<PrincipalsRepository> {
     Arc::new(RwLock::new(HashMap::new()))
}

#[async_trait]
pub trait PrincipalsRepositoryExt {
    async fn with_default_data(self) -> Arc<PrincipalsRepository>;
}

#[async_trait]
impl PrincipalsRepositoryExt for Arc<PrincipalsRepository> {
    async fn with_default_data(self) -> Arc<PrincipalsRepository> {
        let schema = schema();
        let schema_name = schema_name();
        let entity = Entity::from_json_str(USER, Some(&schema)).unwrap();
        let principal = Principal::new(entity, schema_name.clone());
        let key = (principal_type(), user_name());
        self.upsert(key.clone(), principal.clone()).await.unwrap();
        self
    }
}
