use crate::services::backends::kubernetes::common::{
    KubernetesResourceManager, KubernetesResourceManagerConfig, ResourceUpdateHandler,
};
use anyhow::Error;
use k8s_openapi::api::coordination::v1::Lease;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::NamespaceResourceScope;
use kube::core::params::PostParams;
use kube::core::ErrorResponse;
use kube::runtime::reflector::ObjectRef;
use kube::{Api, Client};
use kubert::lease::{ClaimParams, LeaseManager};
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

pub struct LeaseSettings {
    pub claimant: String,
    pub lease_duration: Duration,
    pub renew_deadline: Duration,
    pub lease_name: String,
}

pub struct SynchronizedKubernetesResourceManager<Resource>
where
    Resource: kube::Resource + 'static,
    Resource::DynamicType: Hash + Eq,
{
    resource_manager: KubernetesResourceManager<Resource>,
    api: Api<Lease>,
    lease_settings: LeaseSettings,
}

impl<Resource> SynchronizedKubernetesResourceManager<Resource>
where
    Resource:
        kube::Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    Resource::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(
        resource_manager: KubernetesResourceManager<Resource>,
        api: Api<Lease>,
        lease_settings: LeaseSettings,
    ) -> Self {
        SynchronizedKubernetesResourceManager {
            resource_manager,
            api,
            lease_settings,
        }
    }

    pub async fn replace(&self, name: &str, object: Resource) -> Result<(), Error> {
        let lm = LeaseManager::init(self.api.clone(), self.lease_settings.lease_name.clone()).await?;

        let claims_params = ClaimParams {
            lease_duration: self.lease_settings.lease_duration,
            renew_grace_period: self.lease_settings.renew_deadline,
        };
        lm.ensure_claimed(&self.lease_settings.claimant, &claims_params).await?;
        self.resource_manager.replace(name, object).await?;
        lm.vacate("boxer").await?;
        Ok(())
    }

    pub fn get(&self, object_ref: ObjectRef<Resource>) -> Result<Arc<Resource>, Error> {
        self.resource_manager.get(object_ref)
    }

    pub async fn start(
        config: KubernetesResourceManagerConfig,
        update_handler: Arc<dyn ResourceUpdateHandler<Resource>>,
    ) -> Result<Self, Error> {
        let resource_manager = KubernetesResourceManager::start(config.clone(), update_handler).await?;
        let client = Client::try_from(config.kubeconfig)?;
        let api = Api::<Lease>::namespaced(client, &config.namespace);
        let lease = Lease {
            metadata: ObjectMeta {
                name: Some(config.lease_name.clone()),
                namespace: Some(config.namespace.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = api.create(&PostParams::default(), &lease).await;
        if let Err(e) = err {
            if let kube::Error::Api(ErrorResponse { code: 409, .. }) = e {
                info!("Lease {} already exists, continuing", config.lease_name);
            } else {
                return Err(anyhow::anyhow!("Failed to create lease: {}", e));
            }
        }
        let ls = LeaseSettings {
            claimant: config.claimant.clone(),
            lease_duration: config.lease_duration,
            renew_deadline: config.renew_deadline,
            lease_name: config.lease_name.clone(),
        };
        Ok(SynchronizedKubernetesResourceManager::new(resource_manager, api, ls))
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.resource_manager.stop()
    }

    pub fn namespace(&self) -> String {
        self.resource_manager.namespace.clone()
    }
}
