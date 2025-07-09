// tests module is used to test the repository
#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_principal;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Workaround to use prinltn! for logs.
use std::str::FromStr;
#[cfg(test)]
use std::{println as warn, println as debug};

// Other imports
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::{KubernetesResourceManagerConfig, ResourceUpdateHandler};
use crate::services::base::upsert_repository::{PrincipalIdentity, UpsertRepository};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use cedar_policy::{Entities, EntityUid};
use futures::future;
use futures::future::Ready;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher;
use kube::Resource;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PrincipalData {
    active: String,
    inactive: String,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
struct PrincipalConfigMap {
    metadata: ObjectMeta,
    data: PrincipalData,
}

fn serialize_entities(entities: &Entities) -> anyhow::Result<String> {
    let mut vec = Vec::new(); // Placeholder for JSON serialization, replace with actual schema if needed
    entities.write_to_json(&mut vec)?;
    String::from_utf8(vec).map_err(|e| anyhow!("Failed to serialize entities: {}", e))
}

impl PrincipalConfigMap {
    fn get_active_entities(&self) -> anyhow::Result<Entities> {
        let active_set = Entities::from_json_str(&self.data.active, None)?;
        Ok(active_set)
    }

    fn get_inactive_entities(&self) -> anyhow::Result<Entities> {
        let inactive_set = Entities::from_json_str(&self.data.inactive, None)?;
        Ok(inactive_set)
    }
}

pub struct KubernetesPrincipalRepository {
    resource_manager: SynchronizedKubernetesResourceManager<PrincipalConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesPrincipalRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manager = SynchronizedKubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesPrincipalRepository {
            resource_manager,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_entities(&self, schema: &str) -> anyhow::Result<Arc<PrincipalConfigMap>> {
        let or = ObjectRef::new(schema).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or)
    }

    async fn overwrite(&self, key: PrincipalIdentity, updated_data: PrincipalData) -> Result<(), anyhow::Error> {
        let updated_configmap = PrincipalConfigMap {
            metadata: ObjectMeta {
                name: Some(key.schema_id().clone()),
                namespace: Some(self.resource_manager.namespace().clone()),
                labels: Some(btreemap! {
                    self.label_selector_key.clone() => self.label_selector_value.clone()
                }),
                ..Default::default()
            },
            data: updated_data,
        };
        self.resource_manager
            .replace(&key.schema_id(), updated_configmap)
            .await
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }
}

impl Drop for KubernetesPrincipalRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manager.stop() {
            warn!("Failed to stop KubernetesPrincipalRepository: {}", e);
        }
    }
}

impl TryInto<EntityUid> for &PrincipalIdentity {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<EntityUid, Self::Error> {
        EntityUid::from_str(self.principal_id())
            .map_err(|_| anyhow!("Failed to parse principal ID: {}", self.principal_id()))
    }
}

struct UpdateHandler;
impl ResourceUpdateHandler<PrincipalConfigMap> for UpdateHandler {
    fn handle_update(&self, event: Result<PrincipalConfigMap, watcher::Error>) -> Ready<()> {
        match event {
            Ok(PrincipalConfigMap {
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

#[async_trait]
impl UpsertRepository<PrincipalIdentity, Principal> for KubernetesPrincipalRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: PrincipalIdentity) -> Result<Principal, Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let configmap = self.get_entities(key.schema_id()).await?;
        let active_entities = configmap.get_active_entities()?;
        let entity = active_entities
            .get(&entity_uid)
            .ok_or_else(|| anyhow!("Entity with UID {} not found in active entities", entity_uid))?;

        Ok(Principal::new(entity.clone(), key.schema_id().clone()))
    }

    async fn upsert(&self, key: PrincipalIdentity, principal: Principal) -> Result<(), Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let configmap = self.get_entities(key.schema_id()).await?;

        let inactive = configmap.get_inactive_entities()?;
        if inactive.get(&entity_uid).is_some() {
            bail!(
                "Principal {:?} is inactive in schema {:?}",
                principal.get_entity().uid(),
                principal.get_schema_id()
            )
        }

        let active = configmap
            .get_active_entities()?
            .remove_entities(Some(entity_uid))?
            .add_entities(Some(principal.get_entity().clone()), None)?;

        let updated_data = PrincipalData {
            active: serialize_entities(&active)?,
            inactive: serialize_entities(&inactive)?, // Keep inactive entities unchanged
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn delete(&self, key: PrincipalIdentity) -> Result<(), Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let configmap = self.get_entities(key.schema_id()).await?;

        let active_entities = configmap.get_active_entities()?;

        let to_delete = active_entities
            .get(&entity_uid)
            .ok_or(anyhow!("Entity with UID {} not found in active entities", entity_uid))?;

        let active_entities = active_entities.clone().remove_entities(Some(entity_uid))?;
        let inactive_entities = configmap
            .get_inactive_entities()?
            .add_entities(Some(to_delete.clone()), None)?;

        let updated_data = PrincipalData {
            active: serialize_entities(&active_entities)?,
            inactive: serialize_entities(&inactive_entities)?, // Keep inactive entities unchanged
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn exists(&self, key: PrincipalIdentity) -> Result<bool, Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let active = self
            .get_entities(key.schema_id())
            .await
            .unwrap()
            .get_active_entities()?;
        Ok(active.get(&entity_uid).is_some())
    }
}
