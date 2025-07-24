use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::ResourceUpdateHandler;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use futures::future;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher;
use kube::{Api, CustomResource, Resource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// tests module is used to test the KubernetesIdentityRepository
#[cfg(test)]
mod tests;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};
#[cfg(test)]
use std::{println as warn, println as debug};

use futures::future::Ready;

// Workaround to use prinltn! for logs.
use crate::services::backends::kubernetes::models;
use crate::services::backends::kubernetes::models::base::WithMetadata;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};
use maplit::btreemap;

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
struct IdentitySetData {
    pub active: HashSet<String>,
    pub inactive: HashSet<String>,
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "IdentityProvider",
    plural = "identity-providers",
    singular = "identity-provider",
    namespaced
)]
struct IdentityProviderSpec {
    identities: IdentitySetData,
}

impl Default for IdentityProvider {
    fn default() -> Self {
        IdentityProvider {
            metadata: ObjectMeta::default(),
            spec: IdentityProviderSpec {
                identities: IdentitySetData {
                    inactive: Default::default(),
                    active: Default::default(),
                },
            },
        }
    }
}

impl WithMetadata<ObjectMeta> for IdentityProvider {
    fn with_metadata(mut self, metadata: ObjectMeta) -> Self {
        self.metadata = metadata;
        self
    }
}

pub struct KubernetesIdentityRepository {
    resource_manager: SynchronizedKubernetesResourceManager<IdentityProvider>,
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

    async fn get_identities(&self, provider: &str) -> Result<Arc<IdentityProvider>> {
        let or = ObjectRef::new(provider).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or).ok_or_else(|| {
            anyhow!(
                "Identity provider \"{}\" not found in namespace: {:?}",
                provider,
                self.resource_manager.namespace()
            )
        })
    }

    async fn overwrite(&self, provider: &str, updated_data: &mut IdentityProvider) -> Result<(), anyhow::Error> {
        self.resource_manager.replace(provider, updated_data).await
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
                let mut new_provider = models::empty(name, namespace, labels);
                self.resource_manager.replace(provider, &mut new_provider).await
            }
        }
    }
}

struct UpdateHandler;
impl ResourceUpdateHandler<IdentityProvider> for UpdateHandler {
    fn handle_update(&self, event: core::result::Result<IdentityProvider, watcher::Error>) -> Ready<()> {
        match event {
            Ok(IdentityProvider {
                metadata:
                    ObjectMeta {
                        name: Some(name),
                        namespace: Some(namespace),
                        ..
                    },
                spec: _,
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
        let mut ip = self.get_identities(provider.as_str()).await?;
        if ip.spec.identities.inactive.contains(&user) {
            bail!("User {:?} is inactive in provider {:?}", user, provider)
        }
        let ip = Arc::make_mut(&mut ip);
        ip.spec.identities.active.insert(entity.user_id);
        self.overwrite(&provider, ip).await
    }

    async fn exists(&self, key: (String, String)) -> Result<bool, Self::Error> {
        let (provider, user) = key;
        let contains = self
            .get_identities(provider.as_str())
            .await?
            .spec
            .identities
            .active
            .contains(&user);
        Ok(contains)
    }
}

#[async_trait]
impl ReadOnlyRepository<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: (String, String)) -> Result<ExternalIdentity, Self::ReadError> {
        let (provider, user) = key;
        let active_set = &self.get_identities(&provider).await?.spec.identities.active;

        active_set
            .get(&user)
            .cloned()
            .ok_or(anyhow!("External identity not found: {:?}/{:?}", provider, user))
            .map(|user| ExternalIdentity::new(provider, user))
    }
}

#[async_trait]
impl CanDelete<(String, String), ExternalIdentity> for KubernetesIdentityRepository {
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: (String, String)) -> Result<(), Self::DeleteError> {
        let (provider, user) = key;
        let mut ip = self.get_identities(&provider).await?;
        let mut resource = Arc::make_mut(&mut ip);
        let was_present = resource.spec.identities.active.remove(&user);
        match was_present {
            true => {
                resource.spec.identities.inactive.insert(user.clone());
                self.overwrite(provider.as_str(), resource).await
            }
            false => {
                warn!("User {:?} not found in provider {:?}", user, provider);
                Ok(())
            }
        }
    }
}

impl UpsertRepositoryWithDelete<(String, String), ExternalIdentity> for KubernetesIdentityRepository {}
