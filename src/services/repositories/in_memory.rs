use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::bail;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::RwLock;

#[async_trait]
impl<Entity, Key> UpsertRepository<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug,
{
    type Error = anyhow::Error;

    async fn get(&self, key: Key) -> Result<Entity, Self::Error> {
        let read_guard = self.read().await;
        match (*read_guard).get(&key) {
            Some(entity) => Ok(entity.clone()),
            None => bail!("Entity {:?} not found", key),
        }
    }

    async fn upsert(&self, key: Key, entity: Entity) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).insert(key, entity);
        Ok(())
    }

    async fn delete(&self, key: Key) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).remove(&key);
        Ok(())
    }

    async fn exists(&self, key: Key) -> Result<bool, Self::Error> {
        let read_guard = self.read().await;
        Ok((*read_guard).get(&key).is_some())
    }
}
