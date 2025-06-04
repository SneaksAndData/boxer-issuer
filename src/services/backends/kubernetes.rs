use crate::models::api::external::identity::ExternalIdentity;
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::runtime::WatchStreamExt;
use kube::Resource;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

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
    use std::collections::BTreeMap;
    use super::*;
    use std::sync::Arc;
    use maplit::btreemap;
    use test_context::{test_context, AsyncTestContext};

    struct KubernetesIdentityRepositoryTest {
        api: Arc<Api<ConfigMap>>
    }

    impl AsyncTestContext for KubernetesIdentityRepositoryTest {
        async fn setup() -> KubernetesIdentityRepositoryTest {
            let client = Client::try_default().await.expect("Failed to create Kubernetes client");
            let api: Api<ConfigMap> = Api::default_namespaced(client.clone());
            
            let identities = btreemap! {
                "identity_provider_1" => r#"["user1", "user2"]"#,
                "identity_provider_2" => r#"["user1"]"#,
                "identity_provider_3" => r#"["user4"]"#,
                "identity_provider_4" => r#"[]"#,
                "identity_provider_5" => "",
            }.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

            let config_map = ConfigMap{
                data: Some(identities),
                metadata: ObjectMeta {
                    name: Some("external-identities".to_string()),
                    namespace: Some("default".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            };

            api.create(&PostParams::default(), &config_map).await.expect("Failed to create ConfigMap");
            KubernetesIdentityRepositoryTest {
                api: Arc::new(api),
            }
        }

        async fn teardown(self) {
            self.api.delete("external-identities", &Default::default())
                .await
                .expect("Failed to delete ConfigMap");
        }
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_kubernetes_identity_repository(ctx: &mut KubernetesIdentityRepositoryTest) {
        assert_eq!(true, true);
    }
}
