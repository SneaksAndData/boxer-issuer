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
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::common::{KubernetesRepository, RepositoryConfig, ResourceUpdateHandler};
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
struct PrincipalData {
    active: Vec<Principal>,
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
impl UpsertRepository<String, Principal> for KubernetesPrincipalRepository {
    type Error = anyhow::Error;

    async fn get(&self, schema: String) -> Result<Principal, Self::Error> {
        todo!()
    }

    async fn upsert(&self, key: String, entity: Principal) -> Result<(), Self::Error> {
        todo!()
    }

    async fn delete(&self, key: String) -> Result<(), Self::Error> {
        todo!()
    }

    async fn exists(&self, key: String) -> Result<bool, Self::Error> {
        todo!()
    }
}
