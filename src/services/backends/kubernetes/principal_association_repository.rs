// tests module is used to test the repository
#[cfg(test)]
mod tests;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Workaround to use prinltn! for logs.
#[cfg(test)]
use std::{println as warn, println as debug};

// Other imports
use crate::models::api::external::identity::ExternalIdentity;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::{KubernetesResourceManagerConfig, ResourceUpdateHandler};
use crate::services::base::upsert_repository::{PrincipalIdentity, UpsertRepository};
use anyhow::anyhow;
use async_trait::async_trait;
use futures::future;
use futures::future::Ready;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher;
use kube::Resource;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PrincipalAssociationData {
    active: String,
    inactive: String,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
struct PrincipalAssociationConfigMap {
    metadata: ObjectMeta,
    data: PrincipalAssociationData,
}

impl PrincipalAssociationConfigMap {
    fn get_active_associations(&self) -> anyhow::Result<HashMap<String, PrincipalIdentity>> {
        serde_json::from_str(self.data.active.as_str())
            .map_err(|e| anyhow!("Failed to deserialize active associations: {}", e))
    }

    fn get_inactive_associations(&self) -> anyhow::Result<HashMap<String, PrincipalIdentity>> {
        serde_json::from_str(self.data.inactive.as_str())
            .map_err(|e| anyhow!("Failed to deserialize active associations: {}", e))
    }
}

pub struct KubernetesPrincipalAssociationRepository {
    resource_manager: SynchronizedKubernetesResourceManager<PrincipalAssociationConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesPrincipalAssociationRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manager = SynchronizedKubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesPrincipalAssociationRepository {
            resource_manager,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_entities(&self, key: ExternalIdentity) -> anyhow::Result<Arc<PrincipalAssociationConfigMap>> {
        let name = format!("principals-{}", key.identity_provider);
        let or = ObjectRef::new(&name).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or)
    }

    async fn overwrite(
        &self,
        key: ExternalIdentity,
        updated_data: PrincipalAssociationData,
    ) -> Result<(), anyhow::Error> {
        let name = format!("principals-{}", key.identity_provider);
        let updated_configmap = PrincipalAssociationConfigMap {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(self.resource_manager.namespace().clone()),
                labels: Some(btreemap! {
                    self.label_selector_key.clone() => self.label_selector_value.clone()
                }),
                ..Default::default()
            },
            data: updated_data,
        };
        self.resource_manager
            .replace(&name, updated_configmap)
            .await
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }
}

impl Drop for KubernetesPrincipalAssociationRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manager.stop() {
            warn!("Failed to stop KubernetesPrincipalAssociationRepository: {}", e);
        }
    }
}

struct UpdateHandler;
impl ResourceUpdateHandler<PrincipalAssociationConfigMap> for UpdateHandler {
    fn handle_update(&self, event: Result<PrincipalAssociationConfigMap, watcher::Error>) -> Ready<()> {
        match event {
            Ok(PrincipalAssociationConfigMap {
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

fn to_key(external_identity: &ExternalIdentity) -> String {
    format!("{}/{}", external_identity.identity_provider, external_identity.user_id)
}

#[async_trait]
impl UpsertRepository<ExternalIdentity, PrincipalIdentity> for KubernetesPrincipalAssociationRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: ExternalIdentity) -> Result<PrincipalIdentity, Self::Error> {
        let configmap = self.get_entities(key.clone()).await?;
        let active = configmap.get_active_associations()?;
        let principal_identity = active
            .get(&to_key(&key))
            .ok_or_else(|| anyhow!("Principal with identity {:?} not found in active associations", key))?;
        Ok(principal_identity.clone())
    }

    async fn upsert(&self, key: ExternalIdentity, principal: PrincipalIdentity) -> Result<(), Self::Error> {
        let configmap = self.get_entities(key.clone()).await?;
        let mut active = configmap.get_active_associations()?;
        active.insert(to_key(&key), principal.clone());

        let updated_data = PrincipalAssociationData {
            active: serde_json::to_string(&active)?,
            inactive: configmap.data.inactive.clone(),
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn delete(&self, key: ExternalIdentity) -> Result<(), Self::Error> {
        let configmap = self.get_entities(key.clone()).await?;
        let mut active = configmap.get_active_associations()?;
        let mut inactive = configmap.get_inactive_associations()?;

        let to_delete = active
            .remove(&to_key(&key))
            .ok_or_else(|| anyhow!("Association not found for external identity: {:?}", key))?;

        inactive.insert(to_key(&key), to_delete);
        let updated_data = PrincipalAssociationData {
            active: serde_json::to_string(&active)?,
            inactive: serde_json::to_string(&inactive)?,
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn exists(&self, key: ExternalIdentity) -> Result<bool, Self::Error> {
        let configmap = self.get_entities(key.clone()).await?;
        let active = configmap.get_active_associations()?;
        Ok(active.get(&to_key(&key)).is_some())
    }
}
