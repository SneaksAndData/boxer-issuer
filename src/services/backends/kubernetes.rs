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

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
struct ExternalIdentitiesSet {
    active: HashSet<String>,
    inactive: HashSet<String>,
}

#[derive(Resource, Serialize, Deserialize, Clone)]
#[derive(Debug)]
#[resource(inherit = ConfigMap)]
struct IdentitiesConfigMap {
    metadata: ObjectMeta,
    data: ExternalIdentitiesSet,
}

#[async_trait]
trait KubernetesWriter<T>: Send + Sync
where T: Send + Sync
{ 
    async fn overwrite(&self, object: T) -> Result<IdentitiesConfigMap, anyhow::Error>;
}

#[async_trait]
impl KubernetesWriter<ExternalIdentitiesSet> for Api::<IdentitiesConfigMap> {
    async fn overwrite(&self, ei: ExternalIdentitiesSet) -> Result<IdentitiesConfigMap, anyhow::Error> {
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
    external_identities: ExternalIdentitiesSet,
    writer: Arc<dyn KubernetesWriter<ExternalIdentitiesSet>>,
}

#[async_trait]
impl UpsertRepository<(String, String), ExternalIdentity> for RwLock<KubernetesIdentityRepository> {
    
    type Error = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::Error> {
        todo!()
    }

    async fn upsert(&self, key: (String, String), entity: ExternalIdentity) -> Result<(), Self::Error> {
        todo!()
    }

    async fn delete(&self, key: (String, String)) -> Result<(), Self::Error> {
        todo!()
    }

    async fn exists(&self, key: (String, String)) -> Result<bool, Self::Error> {
        todo!()
    }
}

#[cfg(test)]

/// Tests for KubernetesIdentityRepository
mod tests {
    use std::collections::BTreeMap;
    use super::*;
    use std::sync::Arc;
    use jwt::ToBase64;
    use maplit::{btreemap, hashset};
    use test_context::{test_context, AsyncTestContext};

    struct KubernetesIdentityRepositoryTest {
        api: Arc<Api<ConfigMap>>,
        to_delete: Vec<String>
    }

    impl AsyncTestContext for KubernetesIdentityRepositoryTest {
        async fn setup() -> KubernetesIdentityRepositoryTest {
            let client = Client::try_default().await.expect("Failed to create Kubernetes client");
            let api: Api<ConfigMap> = Api::default_namespaced(client.clone());

            let providers = [
                ( "identity-provider-1", vec!["user1", "user2"], vec![]),
                ( "identity-provider-2", vec!["user1"], vec![]),
                ( "identity-provider-3", vec!["user3"], vec!["user4", "user5"]),
                ( "identity-provider-4", vec![], vec![]),
            ];

            let mut created = Vec::new();
            for (provider, active, inactive) in &providers {
                let data = btreemap! {
                    "active".to_string() => serde_json::to_string(&active).unwrap(),
                    "inactive".to_string() => serde_json::to_string(&inactive).unwrap(),
                };
                let p = provider.to_string();
                let config_map = ConfigMap{
                    data: Some(data),
                    metadata: ObjectMeta {
                        name: Some(p.clone()),
                        namespace: Some("default".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                api.create(&PostParams::default(), &config_map).await.expect("Failed to create ConfigMap");
                created.push(p);
            };

            KubernetesIdentityRepositoryTest {
                api: Arc::new(api),
                to_delete: created,
            }
        }

        async fn teardown(self) {
            for p in self.to_delete {
                let name = p.clone();
                self.api.delete(&name, &Default::default()).await.expect("Failed to delete ConfigMap");
            }
        }
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_kubernetes_identity_repository(ctx: &mut KubernetesIdentityRepositoryTest) {
        assert_eq!(true, true);
    }
}
