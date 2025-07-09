use super::test_data::{schema_fragment, schema_name};
use async_trait::async_trait;
use boxer_issuer::services::base::upsert_repository::SchemaRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn new() -> Arc<SchemaRepository> {
    Arc::new(RwLock::new(HashMap::new()))
}

#[async_trait]
pub trait SchemaRepositoryExt {
    async fn with_default_data(self) -> Arc<SchemaRepository>;
}

#[async_trait]
impl SchemaRepositoryExt for Arc<SchemaRepository> {
    async fn with_default_data(self) -> Arc<SchemaRepository> {
        self.upsert(schema_name().clone(), schema_fragment().clone())
            .await
            .unwrap();
        self
    }
}
