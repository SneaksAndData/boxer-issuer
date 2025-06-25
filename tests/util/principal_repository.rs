use super::test_data::{principal_type, schema, schema_name, user_name, USER};
use async_trait::async_trait;
use boxer_issuer::models::principal::Principal;
use boxer_issuer::services::base::upsert_repository::{PrincipalIdentity, PrincipalRepository};
use cedar_policy::Entity;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn new() -> Arc<PrincipalRepository> {
    Arc::new(RwLock::new(HashMap::new()))
}

#[async_trait]
pub trait PrincipalRepositoryExt {
    async fn with_default_data(self) -> Arc<PrincipalRepository>;
}

#[async_trait]
impl PrincipalRepositoryExt for Arc<PrincipalRepository> {
    async fn with_default_data(self) -> Arc<PrincipalRepository> {
        let schema = schema();
        let schema_name = schema_name();
        let entity = Entity::from_json_str(USER, Some(&schema)).unwrap();
        let principal = Principal::new(entity, schema_name.clone());
        let key = PrincipalIdentity::from((principal_type(), user_name()));
        self.upsert(key.clone(), principal.clone()).await.unwrap();
        self
    }
}
