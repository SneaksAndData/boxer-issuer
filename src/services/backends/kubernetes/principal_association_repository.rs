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
use std::collections::HashMap;
// Other imports
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::common::{KubernetesRepository, RepositoryConfig, ResourceUpdateHandler};
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
use crate::models::api::external::identity::ExternalIdentity;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PrincipalAssociationData {
    active: HashMap<ExternalIdentity, PrincipalIdentity>,
    inactive: HashMap<ExternalIdentity, PrincipalIdentity>,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
struct PrincipalAssociationConfigMap {
    metadata: ObjectMeta,
    data: PrincipalAssociationData,
}

impl PrincipalAssociationConfigMap {
    fn get_active_associations(&self) -> anyhow::Result<HashMap<ExternalIdentity, PrincipalIdentity>> {
        Ok(self.data.active.clone())
    }

    fn get_inactive_associations(&self) -> anyhow::Result<HashMap<ExternalIdentity, PrincipalIdentity>> {
        Ok(self.data.inactive.clone())
    }
}

pub struct KubernetesPrincipalAssociationRepository {
    repository: KubernetesRepository<PrincipalAssociationConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesPrincipalAssociationRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: RepositoryConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let repository = KubernetesRepository::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesPrincipalAssociationRepository {
            repository,
            label_selector_key,
            label_selector_value,
        })
    }

    async fn get_entities(&self, key: ExternalIdentity) -> anyhow::Result<Arc<PrincipalAssociationConfigMap>> {
        let name = format!("principals-{}", key.identity_provider);
        let or = ObjectRef::new(&name).within(self.repository.namespace().as_str());
        self.repository.get(or)
    }

    async fn overwrite(&self, key: PrincipalIdentity, updated_data: PrincipalAssociationData) -> Result<(), anyhow::Error> {
        let updated_configmap = PrincipalAssociationConfigMap {
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
        self.repository
            .replace(&key.schema_id(), updated_configmap)
            .await
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }
}

impl Drop for KubernetesPrincipalAssociationRepository {
    fn drop(&mut self) {
        if let Err(e) = self.repository.stop() {
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

#[async_trait]
impl UpsertRepository<ExternalIdentity, PrincipalIdentity> for KubernetesPrincipalAssociationRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: ExternalIdentity) -> Result<PrincipalIdentity, Self::Error> {
        let configmap = self.get_entities(key).await?;
        let active_entities = configmap.get_active_associations()?;
        let principal_identity = active_entities
            .get(&key)
            .ok_or_else(|| anyhow!("Principal with identity {:?} not found in active associations", key))?;
        Ok(principal_identity.clone())
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
