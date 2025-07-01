use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use anyhow::Error;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::reflector::ObjectRef;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::services::backends::kubernetes::common::{KubernetesRepository, RepositoryConfig, ResourceUpdateHandler};
use k8s_openapi::api::coordination::v1::Lease;
use kube_client::{Api, Client};

pub struct MultithreadResourceManager<Resource>
where
    Resource: kube::Resource + 'static,
    Resource::DynamicType: Hash + Eq,
{
    resource_manager: KubernetesRepository<Resource>,
    api: Api<Lease>
}

impl<Resource> MultithreadResourceManager<Resource>
where
    Resource: kube::Resource<Scope=NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    Resource::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(resource_manager: KubernetesRepository<Resource>, api: Api<Lease>) -> Self {
        MultithreadResourceManager { resource_manager, api }
    }
    
    pub async fn replace(&self, name: &str, object: Resource) -> Result<(), Error> {
        self.resource_manager.replace(name, object).await
    }
    
    pub fn get(&self, object_ref: ObjectRef<Resource>) -> Result<Arc<Resource>, Error> {
        self.resource_manager.get(object_ref)
    }
    
    pub async fn start(config: RepositoryConfig, update_handler: Arc<dyn ResourceUpdateHandler<Resource>>) -> Result<Self, Error> {
        let resource_manager = KubernetesRepository::start(config.clone(), update_handler).await?;
        let client = Client::try_from(config.kubeconfig)?;
        let api = Api::<Lease>::namespaced(client, &config.namespace);
        Ok(MultithreadResourceManager::new(resource_manager, api))
    }
}