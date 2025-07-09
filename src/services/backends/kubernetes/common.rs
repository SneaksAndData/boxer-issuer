#[cfg(test)]
pub mod fixtures;
pub mod synchronized_kubernetes_resource_manager;

use anyhow::{anyhow, Error};
use futures::future::Ready;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::PostParams;
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::watcher::Config;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for the Kubernetes repository.
#[derive(Clone)]
pub struct KubernetesResourceManagerConfig {
    pub namespace: String,
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub kubeconfig: kube::Config,

    pub lease_name: String,
    pub claimant: String,
    pub lease_duration: Duration,
    pub renew_deadline: Duration,
}

impl KubernetesResourceManagerConfig {
    pub fn clone_with_label_selector(&self, label_selector_key: String, label_selector_value: String) -> Self {
        KubernetesResourceManagerConfig {
            namespace: self.namespace.clone(),
            label_selector_key,
            label_selector_value,
            kubeconfig: self.kubeconfig.clone(),
            lease_name: self.lease_name.clone(),
            claimant: self.claimant.clone(),
            lease_duration: self.lease_duration,
            renew_deadline: self.renew_deadline,
        }
    }
}
pub struct KubernetesResourceManager<StoredObject>
where
    StoredObject: Resource + 'static,
    StoredObject::DynamicType: Hash + Eq,
{
    reader: Store<StoredObject>,
    handle: tokio::task::JoinHandle<()>,
    api: Api<StoredObject>,
    namespace: String,
}

pub trait ResourceUpdateHandler<S>: Send + Sync
where
    S: Resource + Send + Sync,
{
    fn handle_update(&self, result: Result<S, watcher::Error>) -> Ready<()>;
}

impl<S> KubernetesResourceManager<S>
where
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(reader: Store<S>, handle: tokio::task::JoinHandle<()>, api: Api<S>, namespace: String) -> Self {
        KubernetesResourceManager {
            reader,
            handle,
            api,
            namespace,
        }
    }

    #[allow(dead_code)]
    pub fn namespace(&self) -> String {
        self.namespace.clone()
    }

    pub async fn replace(&self, _: &str, object: S) -> Result<(), Error> {
        let object_name = object
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Object name is required for replacement"))?;

        let exists = self
            .api
            .get(&object_name)
            .await
            .map(|_| true)
            .or_else(|_| Ok::<bool, Error>(false))?;

        if exists {
            debug!("Replacing existing resource: {}", object_name);
            self.api
                .replace(&object_name, &PostParams::default(), &object)
                .await
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to update resource: {}", e))
        } else {
            debug!("Creating new resource: {}", object_name);
            self.api
                .create(&PostParams::default(), &object)
                .await
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to create resource: {}", e))
        }
    }

    pub fn get(&self, object_ref: ObjectRef<S>) -> Result<Arc<S>, Error> {
        self.reader.get(&object_ref).ok_or_else(|| {
            anyhow!(
                "Object with name [{}] not found in namespace: {:?}",
                object_ref.name,
                object_ref.namespace
            )
        })
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }

    pub async fn start(
        config: KubernetesResourceManagerConfig,
        update_handler: Arc<dyn ResourceUpdateHandler<S>>,
    ) -> anyhow::Result<Self> {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let watcher_config = Config {
            label_selector: Some(format!("{}={}", config.label_selector_key, config.label_selector_value)),
            ..Default::default()
        };
        let stream = watcher(api.clone(), watcher_config);
        let (reader, writer) = reflector::store();

        let reflector = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(move |r| update_handler.handle_update(r));

        let handle = tokio::spawn(reflector);
        reader.wait_until_ready().await?;

        Ok(KubernetesResourceManager::new(reader, handle, api, config.namespace))
    }
}
