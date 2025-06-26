// tests module is used to test the repository
#[cfg(test)]
mod tests;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Workaround to use prinltn! for logs.
#[cfg(test)]
use std::{println as warn, println as debug};
use std::collections::HashSet;
use std::str::FromStr;
// Other imports
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::common::{KubernetesRepository, RepositoryConfig, ResourceUpdateHandler};
use crate::services::base::upsert_repository::{PrincipalIdentity, UpsertRepository};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use cedar_policy::{Entities, EntityUid, SchemaFragment};
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
use utoipa::schema;

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

pub struct KubernetesPrincipalRepository {
    repository: KubernetesRepository<PrincipalConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesPrincipalRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: RepositoryConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let repository = KubernetesRepository::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesPrincipalRepository {
            repository,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_entities(&self, schema: &str) -> anyhow::Result<Arc<PrincipalConfigMap>> {
        let or = ObjectRef::new(schema).within(self.repository.namespace().as_str());
        self.repository.get(or)
    }

    async fn get_active_entities(&self, data: &PrincipalData) -> anyhow::Result<Entities> {
        let active_set = Entities::from_json_str(&data.active, None)?; // TODO: schema?
        Ok(active_set)
    }

    async fn get_inactive_entities(&self, data: &PrincipalData) -> anyhow::Result<Entities> {
        let inactive_set = Entities::from_json_str(&data.inactive, None)?; // TODO: schema?
        Ok(inactive_set)
    }

    async fn overwrite(&self, key: PrincipalIdentity, updated_data: PrincipalData) -> Result<(), anyhow::Error> {
        let updated_configmap = PrincipalConfigMap {
            metadata: ObjectMeta {
                name: Some(key.schema_id().clone()),
                namespace: Some(self.repository.namespace().clone()),
                labels: Some(btreemap! {
                    self.label_selector_key.clone() => self.label_selector_value.clone()
                }),
                ..Default::default()
            },
            data: updated_data,
        };
        self.repository.replace(&key.schema_id(), updated_configmap).await
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }
}

impl Drop for KubernetesPrincipalRepository {
    fn drop(&mut self) {
        if let Err(e) = self.repository.stop() {
            warn!("Failed to stop KubernetesPrincipalRepository: {}", e);
        }
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
        let entity_uid = EntityUid::from_str(key.principal_id()).map_err(|_| {
            anyhow!("Failed to parse principal ID: {}", key.principal_id())
        })?;
        let configmap = self.get_entities(key.schema_id()).await?;
        let active_entities = self.get_active_entities(&configmap.data).await?;
        let entity = active_entities.get(&entity_uid)
            .ok_or_else(|| anyhow!("Entity with UID {} not found in active entities", entity_uid))?;

        Ok(Principal::new(entity.clone(), key.schema_id().clone()))
    }

    async fn upsert(&self, key: PrincipalIdentity, principal: Principal) -> Result<(), Self::Error> {
        let entity_uid = EntityUid::from_str(key.principal_id()).map_err(|_| {
            anyhow!("Failed to parse principal ID: {}", key.principal_id())
        })?;
        let configmap = self.get_entities(key.schema_id()).await?;

        let inactive_set = self.get_inactive_entities(&configmap.data).await?;
        if inactive_set.get(&entity_uid).is_some() {
            bail!("Principal {:?} is inactive in schema {:?}", principal.get_entity().uid(), principal.get_schema_id())
        }

        let active_entities = self.get_active_entities(&configmap.data).await?;
        let mut updated_active = active_entities.iter().filter(|e| e.uid() != entity_uid).map(|c| c.clone()).collect::<Vec<_>>();
        updated_active.push(principal.get_entity().clone());
        let updated_collection = Entities::from_entities(updated_active, None)?;
        let mut vec = Vec::new(); // Placeholder for JSON serialization, replace with actual schema if needed
        updated_collection.write_to_json(&mut vec)?;
        let updated_data = PrincipalData {
            active: String::from_utf8(vec)?,
            inactive: configmap.data.inactive.clone(), // Keep inactive entities unchanged
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn delete(&self, key: PrincipalIdentity) -> Result<(), Self::Error> {
        let entity_uid = EntityUid::from_str(key.principal_id()).map_err(|_| {
            anyhow!("Failed to parse principal ID: {}", key.principal_id())
        })?;
        let configmap = self.get_entities(key.schema_id()).await?;
        let mut inactive_set = self.get_inactive_entities(&configmap.data).await?;
        let mut active_entities = self.get_active_entities(&configmap.data).await?;

        let to_delete = active_entities.get(&entity_uid).ok_or(
            anyhow!("Entity with UID {} not found in active entities", entity_uid)
        )?;
        inactive_set = inactive_set.add_entities(vec![to_delete.clone()], None)?; // TODO: schema?
        active_entities = active_entities.remove_entities(vec![entity_uid])?;

        let mut active_vec = Vec::new(); // Placeholder for JSON serialization, replace with actual schema if needed
        active_entities.write_to_json(&mut active_vec)?;
        let mut inactive_vec = Vec::new(); // Placeholder for JSON serialization, replace with actual schema if needed
        active_entities.write_to_json(&mut inactive_vec)?;

        let updated_data = PrincipalData {
            active: String::from_utf8(active_vec)?,
            inactive: String::from_utf8(inactive_vec)?, // Keep inactive entities unchanged
        };
        self.overwrite(key, updated_data).await?;
        Ok(())
    }

    async fn exists(&self, key: PrincipalIdentity) -> Result<bool, Self::Error> {
        todo!()
    }
}
