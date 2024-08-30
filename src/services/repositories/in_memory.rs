use crate::models::external::identity::{ExternalIdentity, Policy, PolicyAttachment};
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::bail;
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[async_trait]
impl UpsertRepository<ExternalIdentity, (String, String)> for RwLock<HashMap<(String, String), ExternalIdentity>>{
    type Error = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::Error> {
        let read_guard = self.read().await;
        match (*read_guard).get(&key) {
            Some(entity) => Ok(entity.clone()),
            None => bail!("Entity not found"),
        }
    }

    async fn upsert(&mut self, key: (String, String), entity: ExternalIdentity) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).insert(key, entity);
        Ok(())
    }

    async fn delete(&mut self, key: (String, String)) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).remove(&key);
        Ok(())
    }
}

#[async_trait]
impl UpsertRepository<Policy, String> for  RwLock<HashMap<String, Policy>> {
    type Error = anyhow::Error;

    async fn get(&self, key: String) -> Result<Policy, Self::Error> {
        let read_guard = self.read().await;
        match (*read_guard).get(&key) {
            Some(entity) => Ok(entity.clone()),
            None => bail!("Entity not found"),
        }
    }

    async fn upsert(&mut self, key: String, entity: Policy) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).insert(key, entity);
        Ok(())
    }

    async fn delete(&mut self, key: String) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).remove(&key);
        Ok(())
    }
}

#[async_trait]
impl UpsertRepository<PolicyAttachment, ExternalIdentity> for RwLock<HashMap<ExternalIdentity, PolicyAttachment>> {
    type Error = anyhow::Error;

    async fn get(&self, key: ExternalIdentity) -> Result<PolicyAttachment, Self::Error> {
        let read_guard = self.read().await;
        match (*read_guard).get(&key) {
            Some(entity) => Ok(entity.clone()),
            None => bail!("Entity not found"),
        }
    }

    async fn upsert(&mut self, key: ExternalIdentity, entity: PolicyAttachment) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        match (*write_guard).get(&key) {
            Some(entity) => {
                let new_policies = entity.policies.union(&entity.policies).cloned().collect();
                let new_entity = PolicyAttachment {
                        external_identity: entity.external_identity.clone(),
                        policies: new_policies,
                };
                (*write_guard).insert(key, new_entity);
            }
            None => {
                (*write_guard).insert(key, entity);
            }
        }
        Ok(())
    }

    async fn delete(&mut self, key: ExternalIdentity) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).remove(&key);
        Ok(())
    }
}
