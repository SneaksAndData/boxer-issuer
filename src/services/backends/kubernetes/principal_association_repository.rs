// tests module is used to test the repository
#[cfg(test)]
mod tests;

// Other imports
use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::update_handler::UpdateHandler;
use crate::services::backends::kubernetes::models::identity_provider::{IdentityProvider, PrincipalAssociation};
use crate::services::base::upsert_repository::PrincipalIdentity;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use kube::runtime::reflector::ObjectRef;
use log::warn;
use std::collections::HashMap;
use std::sync::Arc;

impl IdentityProvider {
    fn get_active_associations(&self) -> anyhow::Result<HashMap<String, PrincipalIdentity>> {
        let mut hm = HashMap::new();
        for i in &self.spec.identities.active {
            if let Some(ref principal) = i.principal {
                hm.insert(
                    i.name.clone(),
                    PrincipalIdentity::new(principal.schema.clone(), principal.principal.clone()),
                );
            }
        }
        Ok(hm)
    }

    fn associate(&mut self, user: String, principal: PrincipalIdentity) -> anyhow::Result<()> {
        if self.spec.identities.is_deleted(&user) {
            bail!("User {:?} is inactive in provider {:?}", user, self.metadata.name);
        }

        for i in &mut self.spec.identities.active {
            if i.name == user {
                i.principal = Some(PrincipalAssociation {
                    schema: principal.schema_id().clone(),
                    principal: principal.principal_id().clone(),
                })
            }
        }

        Ok(())
    }
}

pub struct KubernetesPrincipalAssociationRepository {
    resource_manager: SynchronizedKubernetesResourceManager<IdentityProvider>,
}

impl KubernetesPrincipalAssociationRepository {
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let resource_manager = SynchronizedKubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesPrincipalAssociationRepository { resource_manager })
    }

    async fn get_entities(&self, provider: &str) -> anyhow::Result<Arc<IdentityProvider>> {
        let or = ObjectRef::new(provider).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or).ok_or_else(|| {
            anyhow!(
                "Identity provider \"{}\" not found in namespace: {:?}",
                provider,
                self.resource_manager.namespace()
            )
        })
    }

    async fn overwrite(
        &self,
        provider: &str,
        updated_data: &mut IdentityProvider,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.resource_manager.replace(provider, updated_data).await
    }
}

impl Drop for KubernetesPrincipalAssociationRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manager.stop() {
            warn!("Failed to stop KubernetesPrincipalAssociationRepository: {}", e);
        }
    }
}

#[async_trait]
impl UpsertRepository<ExternalIdentity, PrincipalIdentity> for KubernetesPrincipalAssociationRepository {
    type Error = anyhow::Error;

    async fn upsert(&self, key: ExternalIdentity, principal: PrincipalIdentity) -> Result<(), Self::Error> {
        let user = key.user_id.clone();
        let mut ip = self.get_entities(&key.identity_provider).await?;
        if ip.spec.identities.is_deleted(&user) {
            bail!("User {:?} is inactive in provider {:?}", user, key.identity_provider)
        }
        let ip = Arc::make_mut(&mut ip);
        ip.associate(key.user_id, principal.clone())?;
        self.overwrite(&key.identity_provider, ip).await
    }

    async fn exists(&self, key: ExternalIdentity) -> Result<bool, Self::Error> {
        let result = self
            .get_entities(&key.identity_provider)
            .await?
            .get_active_associations()?
            .contains_key(&key.user_id);
        Ok(result)
    }
}

#[async_trait]
impl ReadOnlyRepository<ExternalIdentity, PrincipalIdentity> for KubernetesPrincipalAssociationRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: ExternalIdentity) -> Result<PrincipalIdentity, Self::ReadError> {
        self.get_entities(&key.identity_provider)
            .await?
            .get_active_associations()?
            .get(&key.user_id)
            .cloned()
            .ok_or(anyhow::anyhow!("Principal association not found for {:?}", key))
    }
}
