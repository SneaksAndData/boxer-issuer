use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::ResourceUpdateHandler;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use futures::future;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher;
use kube::Resource;
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
use crate::services::backends::kubernetes::models;
use crate::services::backends::kubernetes::models::base::WithMetadata;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};
use maplit::btreemap;
#[cfg(test)]
use std::{println as warn, println as debug};

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

impl Default for IdentitiesConfigMap {
    fn default() -> Self {
        IdentitiesConfigMap {
            metadata: ObjectMeta::default(),
            data: ExternalIdentitiesSet {
                active: "[]".to_string(),
                inactive: "[]".to_string(),
            },
        }
    }
}

impl WithMetadata<ObjectMeta> for IdentitiesConfigMap {
    fn with_metadata(mut self, metadata: ObjectMeta) -> Self {
        self.metadata = metadata;
        self
    }
}

pub struct KubernetesIdentityRepository {
    resource_manager: SynchronizedKubernetesResourceManager<IdentitiesConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesIdentityRepository {
    pub async fn start(config: KubernetesResourceManagerConfig) -> Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manager = SynchronizedKubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesIdentityRepository {
            resource_manager,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_identities(&self, provider: &str) -> Result<Arc<IdentitiesConfigMap>> {
        let or = ObjectRef::new(provider).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or).map_err(|e| {
            anyhow!(
                "Identity provider \"{}\" not found in namespace: {:?}",
                provider,
                self.resource_manager.namespace()
            )
            .context(e)
        })
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
    ) -> Result<(), anyhow::Error> {
        let mut updated_configmap = IdentitiesConfigMap {
            metadata: object_meta.clone(),
            data: updated_data,
        };
        updated_configmap.metadata.resource_version = None;
        self.resource_manager.replace(provider, updated_configmap).await
    }

    pub async fn try_register_identity_provider(&self, provider: &str) -> Result<()> {
        let name = provider.to_string();
        let namespace = self.resource_manager.namespace().clone();
        let labels = btreemap! {
            self.label_selector_key.clone() => self.label_selector_value.clone()
        };
        match self.get_identities(provider).await {
            Ok(_) => Ok(()),
            _ => {
                self.resource_manager
                    .replace(provider, models::empty(name, namespace, labels))
                    .await
            }
        }
    }
}

struct UpdateHandler;
impl ResourceUpdateHandler<IdentitiesConfigMap> for UpdateHandler {
    fn handle_update(&self, event: core::result::Result<IdentitiesConfigMap, watcher::Error>) -> Ready<()> {
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
}

impl Drop for KubernetesIdentityRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manager.stop() {
            warn!("Failed to stop KubernetesIdentityRepository: {}", e);
        }
    }
}

#[async_trait]
impl UpsertRepository<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type Error = anyhow::Error;

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

    async fn exists(&self, key: (String, String)) -> Result<bool, Self::Error> {
        let (provider, user) = key;
        let configmap = self.get_identities(provider.as_str()).await?;
        let active_set = self.get_active_identities(&configmap.data).await?;
        Ok(active_set.contains(&user))
    }
}

#[async_trait]
impl ReadOnlyRepository<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::ReadError> {
        let (provider, user) = key;
        let configmap = self.get_identities(&provider).await?;
        let active_set = self.get_active_identities(&configmap.data).await?;
        let username_extracted = active_set.get(&user).cloned();

        username_extracted
            .ok_or(anyhow!("External identity not found: {:?}/{:?}", provider, user))
            .map(|_| ExternalIdentity::new(provider, user))
    }
}

#[async_trait]
impl CanDelete<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: (String, String)) -> Result<(), Self::DeleteError> {
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
}

impl UpsertRepositoryWithDelete<(String, String), ExternalIdentity> for KubernetesIdentityRepository {}
