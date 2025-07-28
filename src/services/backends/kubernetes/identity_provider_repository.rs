use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use kube::runtime::reflector::ObjectRef;
use std::sync::Arc;

// tests module is used to test the KubernetesIdentityRepository
#[cfg(test)]
mod tests;

// Use log crate when building application
// Workaround to use prinltn! for logs.
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::backends::kubernetes::common::update_handler::UpdateHandler;
use crate::services::backends::kubernetes::models::identity_provider::IdentityProvider;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};
#[cfg(not(test))]
use log::warn;
use maplit::btreemap;
#[cfg(test)]
use std::println as warn;

pub struct KubernetesIdentityProviderRepository {
    label_selector_key: String,
    label_selector_value: String,
    resource_manager: SynchronizedKubernetesResourceManager<IdentityProvider>,
}

impl KubernetesIdentityProviderRepository {
    pub async fn start(config: KubernetesResourceManagerConfig) -> Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manager = SynchronizedKubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesIdentityProviderRepository {
            resource_manager,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_identities(&self, provider: &str) -> Option<Arc<IdentityProvider>> {
        let or = ObjectRef::new(provider).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or)
    }

    async fn overwrite(&self, provider: &str, updated_data: &mut IdentityProvider) -> Result<(), anyhow::Error> {
        self.resource_manager.replace(provider, updated_data).await
    }
}

impl Drop for KubernetesIdentityProviderRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manager.stop() {
            warn!("Failed to stop KubernetesIdentityRepository: {}", e);
        }
    }
}

#[async_trait]
impl UpsertRepository<String, IdentityProviderRegistration> for KubernetesIdentityProviderRepository {
    type Error = anyhow::Error;

    async fn upsert(&self, key: String, entity: IdentityProviderRegistration) -> Result<(), Self::Error> {
        let mut ip = self.get_identities(&key).await.unwrap_or_default();
        let ip = Arc::make_mut(&mut ip);
        ip.metadata.name = Some(key.clone());
        ip.metadata.labels = Some(btreemap! {
            self.label_selector_key.clone() => self.label_selector_value.clone(),
        });
        ip.metadata.namespace = Some(self.resource_manager.namespace().clone());
        ip.spec.oidc = Some(entity.oidc);
        self.overwrite(&key, ip).await
    }

    async fn exists(&self, key: String) -> Result<bool, Self::Error> {
        let contains = self.get_identities(&key).await.is_some();
        Ok(contains)
    }
}

#[async_trait]
impl ReadOnlyRepository<String, IdentityProviderRegistration> for KubernetesIdentityProviderRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: String) -> Result<IdentityProviderRegistration, Self::ReadError> {
        let ip = self
            .get_identities(&key)
            .await
            .ok_or(anyhow!("Identity provider not found"))?;
        match ip.spec.oidc.clone() {
            Some(oidc) => Ok(IdentityProviderRegistration { name: key, oidc }),
            None => bail!("Identity provider {:?} does not have OIDC settings", key),
        }
    }
}

#[async_trait]
impl CanDelete<String, IdentityProviderRegistration> for KubernetesIdentityProviderRepository {
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: String) -> Result<(), Self::DeleteError> {
        self.resource_manager.delete(&key).await
    }
}

impl UpsertRepositoryWithDelete<String, IdentityProviderRegistration> for KubernetesIdentityProviderRepository {}
