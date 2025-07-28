// tests module is used to test the repository
#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_principal;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Other imports
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::common::synchronized_kubernetes_resource_manager::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::common::ResourceUpdateHandler;
use crate::services::backends::kubernetes::models;
use crate::services::backends::kubernetes::models::base::WithMetadata;
use crate::services::base::upsert_repository::PrincipalIdentity;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::{Entities, EntityUid};
use futures::future;
use futures::future::Ready;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use kube::runtime::watcher;
use kube::CustomResource;
use maplit::btreemap;
// Workaround to use prinltn! for logs.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
#[cfg(test)]
use std::{println as warn, println as debug};

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
struct EntitySetData {
    pub active: String,
    pub inactive: String,
}

impl EntitySetData {
    fn get_active_entities(&self) -> anyhow::Result<Entities> {
        if self.active.is_empty() {
            return Ok(Entities::default());
        }
        let active_set = Entities::from_json_str(&self.active, None)?;
        Ok(active_set)
    }

    fn get_inactive_entities(&self) -> anyhow::Result<Entities> {
        if self.inactive.is_empty() {
            return Ok(Entities::default());
        }
        let inactive_set = Entities::from_json_str(&self.inactive, None)?;
        Ok(inactive_set)
    }
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "EntitySet",
    plural = "entity-sets",
    singular = "entity-set",
    namespaced
)]
struct EntitySetSpec {
    entities: EntitySetData,
}

impl Default for EntitySet {
    fn default() -> Self {
        EntitySet {
            metadata: ObjectMeta::default(),
            spec: EntitySetSpec {
                entities: EntitySetData {
                    inactive: Default::default(),
                    active: Default::default(),
                },
            },
        }
    }
}

impl WithMetadata<ObjectMeta> for EntitySet {
    fn with_metadata(mut self, metadata: ObjectMeta) -> Self {
        self.metadata = metadata;
        self
    }
}

fn serialize_entities(entities: &Entities) -> anyhow::Result<String> {
    let mut vec = Vec::new(); // Placeholder for JSON serialization, replace with actual schema if needed
    entities.write_to_json(&mut vec)?;
    String::from_utf8(vec).map_err(|e| anyhow!("Failed to serialize entities: {}", e))
}

pub struct KubernetesPrincipalRepository {
    resource_manager: SynchronizedKubernetesResourceManager<EntitySet>,
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

    async fn get_entities(&self, schema: &str) -> Option<Arc<EntitySet>> {
        let configmap_name = format!("entities-{}", schema);
        let or = ObjectRef::new(&configmap_name).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or)
    }

    async fn overwrite(&self, key: PrincipalIdentity, updated_data: &mut EntitySet) -> Result<(), anyhow::Error> {
        let configmap_name = format!("entities-{}", key.schema_id().clone());
        self.resource_manager.replace(&configmap_name, updated_data).await
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
impl ResourceUpdateHandler<EntitySet> for UpdateHandler {
    fn handle_update(&self, event: Result<EntitySet, watcher::Error>) -> Ready<()> {
        match event {
            Ok(EntitySet {
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

#[async_trait]
impl UpsertRepository<PrincipalIdentity, Principal> for KubernetesPrincipalRepository {
    type Error = anyhow::Error;

    async fn upsert(&self, key: PrincipalIdentity, principal: Principal) -> Result<(), Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let name = format!("entities-{}", key.schema_id().clone());
        let namespace = self.resource_manager.namespace().clone();
        let labels = btreemap! {
            self.label_selector_key.clone() => self.label_selector_value.clone()
        };

        let mut resource = self
            .get_entities(key.schema_id())
            .await
            .unwrap_or(Arc::new(models::empty(name, namespace, labels)));
        let resource = Arc::make_mut(&mut resource);
        let inactive = resource.spec.entities.get_inactive_entities()?;

        if inactive.get(&entity_uid).is_some() {
            bail!(
                "Principal {:?} is inactive in schema {:?}",
                principal.get_entity().uid(),
                principal.get_schema_id()
            )
        }

        let new_active = resource
            .spec
            .entities
            .get_active_entities()?
            .remove_entities(Some(entity_uid))?
            .add_entities(Some(principal.get_entity().clone()), None)?;

        resource.spec.entities.active = serialize_entities(&new_active)?;

        self.overwrite(key, resource).await
    }

    async fn exists(&self, key: PrincipalIdentity) -> Result<bool, Self::Error> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let active = self
            .get_entities(key.schema_id())
            .await
            .ok_or(anyhow!("Cannot fin entities for schema {}", key.schema_id()))?;
        let active = active.spec.entities.get_active_entities()?;
        for entity in active.iter() {
            debug!("Found active entity: {:?}", entity.uid());
        }
        Ok(active.get(&entity_uid).is_some())
    }
}

#[async_trait]
impl ReadOnlyRepository<PrincipalIdentity, Principal> for KubernetesPrincipalRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: PrincipalIdentity) -> Result<Principal, Self::ReadError> {
        let entity_uid: EntityUid = (&key).try_into()?;
        let resource = self
            .get_entities(key.schema_id())
            .await
            .ok_or(anyhow!("Cannot fin entities for schema {}", key.schema_id()))?;
        let active_entities = resource.spec.entities.get_active_entities()?;
        for entity in active_entities.clone() {
            debug!("Found active entity: {:?}", entity.uid());
        }
        let entity = active_entities
            .get(&entity_uid)
            .ok_or_else(|| anyhow!("Entity with UID {} not found in active entities", entity_uid))?;
        Ok(Principal::new(entity.clone(), key.schema_id().clone()))
    }
}
