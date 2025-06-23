use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::reflector::Store;
use crate::services::backends::kubernetes::watcher::Config;
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::{anyhow, bail, Error, Result};
use async_trait::async_trait;
use futures::{future, StreamExt};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::Resource;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

#[cfg(not(test))]
use log::{debug, warn}; // Use log crate when building application

#[cfg(test)]
use std::{println as warn, println as debug}; // Workaround to use prinltn! for logs.

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
    reader: Store<IdentitiesConfigMap>,
    handle: tokio::task::JoinHandle<()>,
    api: Api<IdentitiesConfigMap>,
    namespace: String,
}

impl KubernetesIdentityRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in tests
    async fn start(namespace: &str) -> Result<Self> {
        let client = Client::try_default().await?;
        let api: Api<IdentitiesConfigMap> = Api::namespaced(client.clone(), namespace);
        let config = Config {
            label_selector: Some("app=identity-provider".to_string()),
            ..Default::default()
        };
        let stream = watcher(api.clone(), config);
        let (reader, writer) = reflector::store();

        // reader.wait_until_ready().await?;

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
        reader.wait_until_ready().await?;
        Ok(KubernetesIdentityRepository {
            reader,
            handle,
            api,
            namespace: namespace.to_string(),
        })
    }

    fn stop(&self) -> Result<()> {
        self.handle.abort();
        debug!("KubernetesIdentityRepository stopped");
        Ok(())
    }

    async fn get_identities(&self, provider: &str) -> Result<Arc<IdentitiesConfigMap>> {
        let or = ObjectRef::new(provider).within(self.namespace.as_str());
        self.reader.get(&or).ok_or(anyhow!(
            "Identity provider \"{}\" not found in namespace: {:?}",
            provider,
            or.namespace
        ))
    }

    async fn get_active_identities(&self, ids: &ExternalIdentitiesSet) -> Result<HashSet<String>> {
        let active_set: HashSet<String> =
            serde_json::from_str(&ids.active).map_err(|e| anyhow!("Failed to parse active identities: {}", e))?;
        Ok(active_set)
    }

    async fn get_inactive_identities(&self, ids: &ExternalIdentitiesSet) -> Result<HashSet<String>> {
        let active_set: HashSet<String> =
            serde_json::from_str(&ids.inactive).map_err(|e| anyhow!("Failed to parse active identities: {}", e))?;
        Ok(active_set)
    }

    async fn overwrite(
        &self,
        provider: &str,
        object_meta: ObjectMeta,
        updated_data: ExternalIdentitiesSet,
    ) -> Result<(), Error> {
        let updated_configmap = IdentitiesConfigMap {
            metadata: object_meta.clone(),
            data: updated_data,
        };

        self.api
            .replace(&provider, &PostParams::default(), &updated_configmap)
            .await
            .map(|_| ())
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }
}

impl Drop for KubernetesIdentityRepository {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            warn!("Failed to stop KubernetesIdentityRepository: {}", e);
        }
    }
}

#[async_trait]
impl UpsertRepository<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::Error> {
        let (provider, user) = key;
        let username_extracted = self.api.get(provider.as_str()).await.map(|cm| {
            let set: HashSet<String> = serde_json::from_str(&cm.data.active).ok()?;
            set.get(user.as_str()).cloned()
        })?;

        username_extracted
            .ok_or(anyhow!("External identity not found: {:?}/{:?}", provider, user))
            .map(|_| ExternalIdentity::new(provider, user))
    }

    async fn upsert(&self, key: (String, String), entity: ExternalIdentity) -> Result<(), Self::Error> {
        let (provider, user) = key;
        let configmap = self.get_identities(provider.as_str()).await?;
        let inactive_set = self.get_inactive_identities(&configmap.data).await?;
        if inactive_set.contains(&user) {
            bail!("User {:?} is inactive in provider {:?}", user, provider)
        }

        let mut active_set = self.get_inactive_identities(&configmap.data).await?;

        active_set.insert(entity.user_id);
        let updated_data = ExternalIdentitiesSet {
            active: serde_json::to_string(&active_set)?,
            inactive: serde_json::to_string(&inactive_set)?,
        };

        self.overwrite(provider.as_str(), configmap.metadata.clone(), updated_data)
            .await
    }

    async fn delete(&self, key: (String, String)) -> Result<(), Self::Error> {
        let (provider, user) = key;
        let configmap = self.get_identities(provider.as_str()).await?;
        let mut active_set = self.get_active_identities(&configmap.data).await?;

        let was_present = active_set.remove(&user);
        if was_present {
            let mut inactive_set = self.get_inactive_identities(&configmap.data).await?;
            inactive_set.insert(user.clone());
            let updated_data = ExternalIdentitiesSet {
                active: serde_json::to_string(&active_set)?,
                inactive: serde_json::to_string(&inactive_set)?,
            };
            self.overwrite(provider.as_str(), configmap.metadata.clone(), updated_data)
                .await
        } else {
            Ok(())
        }
    }

    async fn exists(&self, key: (String, String)) -> Result<bool, Self::Error> {
        let (provider, user) = key;
        let configmap = self.get_identities(provider.as_str()).await?;
        let active_set = self.get_active_identities(&configmap.data).await?;
        Ok(active_set.contains(&user))
    }
}

#[cfg(test)]

/// Tests for KubernetesIdentityRepository
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::Namespace;
    use maplit::btreemap;
    use serde_json::json;
    use std::println as info;
    use std::sync::Arc;
    use test_context::{test_context, AsyncTestContext};
    use uuid::Uuid;

    #[allow(dead_code)] // Dead code is allowed here because this struct is used in tests
    struct KubernetesIdentityRepositoryTest {
        api: Arc<Api<ConfigMap>>,
        repository: Arc<KubernetesIdentityRepository>,
    }

    impl AsyncTestContext for KubernetesIdentityRepositoryTest {
        async fn setup() -> KubernetesIdentityRepositoryTest {
            let client = Client::try_default().await.expect("Failed to create Kubernetes client");

            let namespace = Uuid::new_v4().to_string();
            info!("Using namespace: {}", namespace);
            let namespaces: Api<Namespace> = Api::all(client.clone());
            let ns = serde_json::from_value(json!({ "metadata": { "name": namespace.clone() } }))
                .expect("Failed to deserialize namespace");
            namespaces
                .create(&PostParams::default(), &ns)
                .await
                .expect("Create Namespace failed");

            let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace.as_str());

            let providers = [
                ("identity-provider-1", vec!["user1", "user2"], vec![]),
                ("identity-provider-2", vec!["user1"], vec![]),
                (
                    "identity-provider-3",
                    vec!["user3"],
                    vec!["user4", "user5", "deleted_user"],
                ),
                ("identity-provider-4", vec![], vec![]),
            ];

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
                        namespace: Some(namespace.clone()),
                        labels: Some(btreemap! {
                            "app".to_string() => "identity-provider".to_string(),
                            "provider".to_string() => provider.to_string(),
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                api.create(&PostParams::default(), &config_map)
                    .await
                    .expect("Failed to create ConfigMap");
            }

            let repository = KubernetesIdentityRepository::start(namespace.clone().as_str())
                .await
                .expect("Failed to start repository");

            KubernetesIdentityRepositoryTest {
                api: Arc::new(api),
                repository: Arc::new(repository),
            }
        }

        async fn teardown(self) {
            // do nothing
        }
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-1".to_string();
        let user = "user1".to_string();

        // Act
        let external_identity = ctx
            .repository
            .get((provider.clone(), user.clone()))
            .await
            .expect("Failed to get external identity");

        // Assert
        assert_eq!(external_identity.clone().user_id, "user1");
        assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_not_existing_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-1".to_string();
        let user = "user3".to_string();

        // Act
        let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(external_identity.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_unexisted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-2".to_string();
        let user = "i_do_not_exist".to_string();

        // Act
        let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(external_identity.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-3".to_string();
        let user = "deleted_user".to_string();

        // Act
        let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(external_identity.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_from_empty_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-4".to_string();
        let user = "user1".to_string();

        // Act
        let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(external_identity.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_get_from_not_existed_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-5".to_string();
        let user = "user1".to_string();

        // Act
        let external_identity = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(external_identity.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_add_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-1".to_string();
        let user = "new_user".to_string();
        let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

        let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
        // Assert that the user does not exist before upsert
        assert_eq!(old_state.ok(), None);

        // Act
        ctx.repository
            .upsert((provider.clone(), user.clone()), external_identity)
            .await
            .expect("Failed to upsert external identity");

        // Assert
        let external_identity = ctx
            .repository
            .get((provider.clone(), user.clone()))
            .await
            .expect("Failed to get external identity");

        assert_eq!(external_identity.clone().user_id, "new_user");
        assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_add_to_unexisted_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-5".to_string();
        let user = "new_user".to_string();
        let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

        // Act
        let result = ctx
            .repository
            .upsert((provider.clone(), user.clone()), external_identity)
            .await;

        // Assert
        let message = result.err().unwrap().to_string();
        assert!(
            message.contains("Identity provider \"identity-provider-5\" not found"),
            "Unexpected error message: {}",
            message
        );
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_add_duplicate(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-1".to_string();
        let user = "user1".to_string();
        let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

        let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
        // Assert that the user does not exist before upsert
        assert_eq!(
            old_state.ok(),
            Some(ExternalIdentity::new(provider.clone(), user.clone()))
        );

        // Act
        ctx.repository
            .upsert((provider.clone(), user.clone()), external_identity)
            .await
            .expect("Failed to upsert external identity");

        // Assert
        let external_identity = ctx
            .repository
            .get((provider.clone(), user.clone()))
            .await
            .expect("Failed to get external identity");

        assert_eq!(external_identity.clone().user_id, "user1");
        assert_eq!(external_identity.clone().identity_provider, "identity-provider-1");
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_add_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-3".to_string();
        let user = "deleted_user".to_string();
        let external_identity = ExternalIdentity::new(provider.clone(), user.clone());

        let old_state = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert that the user does not exist before upsert
        assert_eq!(old_state.ok(), None);

        // Act
        let result = ctx
            .repository
            .upsert((provider.clone(), user.clone()), external_identity)
            .await;

        let message = result.err().unwrap().to_string();
        // Assert
        assert!(
            message.contains("User \"deleted_user\" is inactive in provider \"identity-provider-3\""),
            "Unexpected error message: {}",
            message
        );
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_delete_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-1".to_string();
        let user = "user1".to_string();

        let old_state = ctx.repository.get((provider.clone(), user.clone())).await;
        // Assert that the user exists before delete
        assert_eq!(
            old_state.ok(),
            Some(ExternalIdentity::new(provider.clone(), user.clone()))
        );

        // Act
        ctx.repository
            .delete((provider.clone(), user.clone()))
            .await
            .expect("Failed to delete external identity");

        let new_state = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(new_state.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_delete_deleted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-3".to_string();
        let user = "deleted_user".to_string();

        // Act
        ctx.repository
            .delete((provider.clone(), user.clone()))
            .await
            .expect("Failed to delete external identity");

        let new_state = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(new_state.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_delete_unexisted_user(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-4".to_string();
        let user = "i_do_not_exist".to_string();

        // Act
        ctx.repository
            .delete((provider.clone(), user.clone()))
            .await
            .expect("Failed to delete external identity");

        let new_state = ctx.repository.get((provider.clone(), user.clone())).await;

        // Assert
        assert_eq!(new_state.ok(), None);
    }

    #[test_context(KubernetesIdentityRepositoryTest)]
    #[tokio::test]
    async fn test_delete_from_unexisted_provider(ctx: &mut KubernetesIdentityRepositoryTest) {
        // Arrange
        let provider = "identity-provider-5".to_string();
        let user = "user1".to_string();

        // Act
        let result = ctx.repository.delete((provider.clone(), user.clone())).await;

        // Assert
        let message = result.err().unwrap().to_string();
        assert!(
            message.contains("Identity provider \"identity-provider-5\" not found"),
            "Unexpected error message: {}",
            message
        );
    }
}
