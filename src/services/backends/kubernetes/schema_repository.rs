// tests module is used to test the repository
#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_schema;

#[cfg(test)]
mod test_reduced_schema;

// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Workaround to use prinltn! for logs.
#[cfg(test)]
use std::{println as warn, println as debug};

// Other imports
use crate::services::backends::kubernetes::common::{
    KubernetesResourceManager, KubernetesResourceManagerConfig, ResourceUpdateHandler,
};
use crate::services::base::upsert_repository::UpsertRepository;
use anyhow::anyhow;
use async_trait::async_trait;
use cedar_policy::SchemaFragment;
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
struct SchemaData {
    active: String,
    content: String,
}

impl TryInto<SchemaFragment> for SchemaData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<SchemaFragment, Self::Error> {
        SchemaFragment::from_json_str(self.content.as_str()).map_err(|err| anyhow!("{}", err))
    }
}

impl TryFrom<SchemaFragment> for SchemaData {
    type Error = anyhow::Error;

    fn try_from(schema: SchemaFragment) -> Result<Self, Self::Error> {
        let serialized = schema
            .to_json_value()
            .map_err(|err| anyhow!("Failed to convert schema to JSON string: {}", err))?;
        Ok(SchemaData {
            active: "true".to_string(),
            content: serde_json::to_string_pretty(&serialized)?,
        })
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
struct SchemaConfigMap {
    metadata: ObjectMeta,
    data: SchemaData,
}

pub struct KubernetesSchemaRepository {
    resource_manger: KubernetesResourceManager<SchemaConfigMap>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesSchemaRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manger = KubernetesResourceManager::start(config, Arc::new(UpdateHandler)).await?;
        Ok(KubernetesSchemaRepository {
            resource_manger,
            label_selector_key,
            label_selector_value,
        })
    }
}

impl Drop for KubernetesSchemaRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manger.stop() {
            warn!("Failed to stop KubernetesSchemaRepository: {}", e);
        }
    }
}

struct UpdateHandler;
impl ResourceUpdateHandler<SchemaConfigMap> for UpdateHandler {
    fn handle_update(&self, event: Result<SchemaConfigMap, watcher::Error>) -> Ready<()> {
        match event {
            Ok(SchemaConfigMap {
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
impl UpsertRepository<String, SchemaFragment> for KubernetesSchemaRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: String) -> Result<SchemaFragment, Self::Error> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        let resource_object = self.resource_manger.get(or).map_err(|e| anyhow!(e))?;
        if resource_object.data.active.contains("false") {
            return Err(anyhow!("Schema is not active"));
        }
        let result: SchemaFragment = resource_object.data.clone().try_into()?;
        Ok(result)
    }

    async fn upsert(&self, key: String, entity: SchemaFragment) -> Result<(), Self::Error> {
        let updated_configmap = SchemaConfigMap {
            metadata: ObjectMeta {
                name: Some(key.clone()),
                namespace: Some(self.resource_manger.namespace().clone()),
                labels: Some(btreemap! {
                    self.label_selector_key.clone() => self.label_selector_value.clone()
                }),
                ..Default::default()
            },
            data: entity.try_into()?,
        };
        self.resource_manger
            .replace(&key, updated_configmap)
            .await
            .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))
    }

    async fn delete(&self, key: String) -> Result<(), Self::Error> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        let mut resource_ref = self.resource_manger.get(or).map_err(|e| anyhow!(e))?;
        if resource_ref.data.active.contains("false") {
            return Ok(());
        }
        let resource_object = Arc::make_mut(&mut resource_ref);
        resource_object.data.active = "false".to_string();
        self.resource_manger.replace(&key, resource_object.clone()).await
    }

    async fn exists(&self, key: String) -> Result<bool, Self::Error> {
        let or: ObjectRef<SchemaConfigMap> =
            ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        self.resource_manger.get(or).map(|_| true).or_else(|e| {
            if e.to_string().contains("not found") {
                Ok(false)
            } else {
                Err(anyhow!(e))
            }
        })
    }
}
