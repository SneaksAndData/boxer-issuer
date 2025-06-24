use crate::models::api::external::identity::ExternalIdentity;
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::{anyhow, bail, Error, Result};
use async_trait::async_trait;
use futures::{future, StreamExt};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::Resource;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

// tests module is used to test the KubernetesIdentityRepository
#[cfg(test)]
mod tests;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

use futures::future::Ready;

// Workaround to use prinltn! for logs.
use kube::runtime::watcher::Config;
#[cfg(test)]
use std::{println as warn, println as debug};

/// Configuration for the Kubernetes identity repository.
pub struct RepositoryConfig {
    pub namespace: String,
    pub label_selector_key: String,
    pub label_selector_value: String,
}

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
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    async fn start(config: RepositoryConfig) -> Result<Self> {
        let client = Client::try_default().await?;
        let api: Api<IdentitiesConfigMap> = Api::namespaced(client.clone(), config.namespace.as_str());
        let watcher_config = Config {
            label_selector: Some(format!("{}={}", config.label_selector_key, config.label_selector_value)),
            ..Default::default()
        };
        let stream = watcher(api.clone(), watcher_config);
        let (reader, writer) = reflector::store();

        let reflector = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(|r| Self::handle_event(r));

        let handle = tokio::spawn(reflector);
        reader.wait_until_ready().await?;
        Ok(KubernetesIdentityRepository {
            reader,
            handle,
            api,
            namespace: config.namespace,
        })
    }

    fn handle_event(event: core::result::Result<IdentitiesConfigMap, watcher::Error>) -> Ready<()> {
        match event {
            Ok(IdentitiesConfigMap {
                metadata:
                    ObjectMeta {
                        name: Some(name),
                        namespace: Some(namespace),
                        ..
                    },
                data: _,
            }) => debug!("Saw [{}] in [{}]", name, namespace),
            Ok(_) => warn!("Saw an object without name or namespace"),
            Err(e) => warn!("watcher error: {}", e),
        }
        future::ready(())
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

        let configmap = self.get_identities(&provider).await?;
        let active_set = self.get_active_identities(&configmap.data).await?;
        let username_extracted = active_set.get(&user).cloned();

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
