use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::reflector::Store;
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use config::Config;
use futures::{future, FutureExt, Stream, StreamExt};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::Resource;
use kube::{Api, Client};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ExternalIdentitiesSet {
    active: String,
    inactive: String,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
struct IdentitiesConfigMap {
    metadata: ObjectMeta,
    data: ExternalIdentitiesSet,
}

struct KubernetesIdentityRepository {
    external_identities: ExternalIdentitiesSet,
    reader: Store<IdentitiesConfigMap>,
    handle: tokio::task::JoinHandle<()>,
    api: Api<IdentitiesConfigMap>,
}

impl KubernetesIdentityRepository {
    async fn start() -> Result<Self> {
        let client = Client::try_default().await?;
        let api: Api<IdentitiesConfigMap> = Api::default_namespaced(client.clone());
        let stream = watcher(api.clone(), Default::default());
        let (reader, writer) = reflector::store();
        let rf = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(|r| {
                future::ready(match r {
                    Ok(o) => debug!("Saw {} in {}", o.metadata.name.unwrap(), o.metadata.namespace.unwrap()), // TODO: remove unwraps
                    Err(e) => warn!("watcher error: {e}"),
                })
            });

        let handle = tokio::spawn(rf); // poll forever
        Ok(KubernetesIdentityRepository {
            external_identities: ExternalIdentitiesSet {
                active: "".to_string(),
                inactive: "".to_string()
            },
            reader,
            handle,
            api,
        })
    }
}

#[async_trait]
impl UpsertRepository<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::Error> {
        let (provider, user) = key;
        let username_extracted = self
            .api
            .get(provider.as_str())
            .await
            .map(|cm| {
                let set: HashSet<String> = serde_json::from_str(&cm.data.active).ok()?;
                set.get(user.as_str()).cloned()
            })?;

        username_extracted
            .ok_or(anyhow!("External identity not found: {:?}/{:?}", provider, user))
            .map(|_| ExternalIdentity::new(provider, user))
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
    use super::*;
    use maplit::btreemap;
    use std::sync::Arc;
    use test_context::{test_context, AsyncTestContext};

    struct KubernetesIdentityRepositoryTest {
        api: Arc<Api<ConfigMap>>,
        to_delete: Vec<String>,
    }

    impl AsyncTestContext for KubernetesIdentityRepositoryTest {
        async fn setup() -> KubernetesIdentityRepositoryTest {
            let client = Client::try_default().await.expect("Failed to create Kubernetes client");
            let api: Api<ConfigMap> = Api::default_namespaced(client.clone());

            let providers = [
                ("identity-provider-1", vec!["user1", "user2"], vec![]),
                ("identity-provider-2", vec!["user1"], vec![]),
                ("identity-provider-3", vec!["user3"], vec!["user4", "user5"]),
                ("identity-provider-4", vec![], vec![]),
            ];

            let mut created = Vec::new();
            for (provider, active, inactive) in &providers {
                let data = btreemap! {
                    "active".to_string() => serde_json::to_string(&active).unwrap(),
                    "inactive".to_string() => serde_json::to_string(&inactive).unwrap(),
                };
                let p = provider.to_string();
                let config_map = ConfigMap {
                    data: Some(data),
                    metadata: ObjectMeta {
                        name: Some(p.clone()),
                        namespace: Some("default".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                api.create(&PostParams::default(), &config_map)
                    .await
                    .expect("Failed to create ConfigMap");
                created.push(p);
            }

            KubernetesIdentityRepositoryTest {
                api: Arc::new(api),
                to_delete: created,
            }
        }

        async fn teardown(self) {
            for p in self.to_delete {
                let name = p.clone();
                self.api
                    .delete(&name, &Default::default())
                    .await
                    .expect("Failed to delete ConfigMap");
            }
        }
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        let repository = KubernetesIdentityRepository::start().await.expect("Failed to start repository");
        let provider = "identity-provider-1".to_string();
        let user = "user1".to_string();
        let external_identity = repository
            .get((provider.clone(), user.clone()))
            .await;
        assert_eq!(external_identity.unwrap().user_id, "user1");
    }
    
    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_not_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        let repository = KubernetesIdentityRepository::start().await.expect("Failed to start repository");
        let provider = "identity-provider-1".to_string();
        let user = "user3".to_string();
        let external_identity = repository
            .get((provider.clone(), user.clone()))
            .await;
        assert_eq!(external_identity.ok(), None);
    }
}
