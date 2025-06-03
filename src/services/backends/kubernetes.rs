use crate::models::api::external::identity::ExternalIdentity;
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::Resource;

type ExternalIdentities = HashMap<String, HashSet<String>>;

#[derive(Resource, Serialize, Deserialize, Clone)]
#[derive(Debug)]
#[resource(inherit = ConfigMap)]
struct IdentitiesConfigMap {
    metadata: ObjectMeta,
    data: ExternalIdentities,
}

#[async_trait]
trait KubernetesWriter<T>: Send + Sync
where T: Send + Sync
{ 
    async fn overwrite(&self, object: T) -> Result<IdentitiesConfigMap, anyhow::Error>;
}

#[async_trait]
impl KubernetesWriter<ExternalIdentities> for Api::<IdentitiesConfigMap> {
    async fn overwrite(&self, ei: ExternalIdentities) -> Result<IdentitiesConfigMap, anyhow::Error> {
        let object = IdentitiesConfigMap {
            metadata: ObjectMeta {
                name: Some("external-identities".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            data: ei,
        };
        let id = self.create(&PostParams::default(), &object).await?;
        Ok(id)
    }
}

struct KubernetesIdentityRepository {
    current_version: String,
    external_identities: ExternalIdentities,
    writer: Arc<dyn KubernetesWriter<ExternalIdentities>>,
}

#[async_trait]
impl UpsertRepository<(String, String), ExternalIdentity> for RwLock<KubernetesIdentityRepository> {
    
    type Error = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::Error> {
        let (identity_provider, identity) = key;
        let read_guard = self.read().await;
        match (*read_guard).external_identities.get(&identity_provider) {
            Some(entity) => Ok(
                if entity.contains(&identity_provider) {
                    ExternalIdentity::new(identity_provider, identity)
                } 
                else {
                    bail!("Identity {:?} not found for provider {:?}", identity, identity_provider)
                }
            ),
            None => bail!("Identity provider not found: {:?}", identity_provider),
        }
    }

    async fn upsert(&self, key: (String, String), entity: ExternalIdentity) -> Result<(), Self::Error> {
        let (identity_provider, identity) = key;
        let mut write_guard = self.write().await;
        let identtities = (*write_guard).external_identities.entry(identity_provider.clone()).or_insert_with(HashSet::new);
        identtities.insert(identity.clone());
        (*write_guard).writer.overwrite((*write_guard).external_identities.clone()).await?;
        Ok(())
    }

    async fn delete(&self, key: (String, String)) -> Result<(), Self::Error> {
        let (identity_provider, identity) = key;
        let mut write_guard = self.write().await;
        let identtities = (*write_guard).external_identities.entry(identity_provider.clone()).or_insert_with(HashSet::new);
        identtities.insert(identity.clone());
        (*write_guard).writer.overwrite((*write_guard).external_identities.clone()).await?;
        Ok(())
    }

    async fn exists(&self, key: (String, String)) -> Result<bool, Self::Error> {
        let (identity_provider, identity) = key;
        let read_guard = self.read().await;
        match (*read_guard).external_identities.get(&identity_provider) {
            Some(entity) => Ok(
                return Ok(entity.contains(&identity_provider))
            ),
            None => bail!("Identity provider not found: {:?}", identity_provider),
        }
    }
}

#[cfg(test)]

/// Tests for KubernetesIdentityRepository
mod tests {
    use super::*;
    use crate::services::base::upsert_repository::UpsertRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_kubernetes_identity_repository() {
        let client = Client::try_default().await.unwrap();
        let api: Api<IdentitiesConfigMap> = Api::default_namespaced(client);
        let writer = Arc::new(api);

        let repository = RwLock::new(KubernetesIdentityRepository {
            current_version: "v1".to_string(),
            external_identities: HashMap::new(),
            writer,
        });

        let identity = ExternalIdentity::new("provider1".to_string(), "identity1".to_string());
        repository.upsert(("provider1".to_string(), "identity1".to_string()), identity.clone()).await.unwrap();
        
        assert!(repository.exists(("provider1".to_string(), "identity1".to_string())).await.unwrap());
        
        let fetched_identity = repository.get(("provider1".to_string(), "identity1".to_string())).await.unwrap();
        assert_eq!(fetched_identity, identity);
        
        repository.delete(("provider1".to_string(), "identity1".to_string())).await.unwrap();
        assert!(!repository.exists(("provider1".to_string(), "identity1".to_string())).await.unwrap());
    }
}
