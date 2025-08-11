pub mod identity_provider_repository;
pub mod identity_repository;
pub mod principal_repository;

mod kubernetes_validator_provider;

use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;

use crate::services::backends::base::IssuerBackend;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use crate::services::configuration::models::{BackendSettings, KubernetesBackendSettings};
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::from_cluster;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, ListenerConfig, UpdateLabels,
};
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::kubernetes::repositories::{KubernetesRepository, SoftDeleteResource};
use boxer_core::services::backends::{Backend, BackendConfiguration};
use boxer_core::services::service_provider::ServiceProvider;
use k8s_openapi::NamespaceResourceScope;
use kube::config::Kubeconfig;
use kube::Config;
use kubernetes_validator_provider::KubernetesValidatorProvider;
use log::{debug, info};
use std::hash::Hash;
use std::process::Command;
use std::sync::Arc;

pub struct KubernetesBackend {
    pub schemas_repository: Option<Arc<SchemaRepository>>,
    pub entities_repository: Option<Arc<PrincipalRepository>>,
    pub identity_repository: Option<Arc<IdentityRepository>>,
    pub identity_provider_repository: Option<Arc<IdentityProviderRepository>>,
    pub validator_provider: Option<Arc<KubernetesValidatorProvider>>,
}

impl KubernetesBackend {
    pub fn new() -> Self {
        KubernetesBackend {
            schemas_repository: None,
            entities_repository: None,
            identity_repository: None,
            identity_provider_repository: None,
            validator_provider: None,
        }
    }
}

impl ServiceProvider<Arc<SchemaRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<SchemaRepository> {
        self.schemas_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl ServiceProvider<Arc<PrincipalRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<PrincipalRepository> {
        self.entities_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl ServiceProvider<Arc<IdentityRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<IdentityRepository> {
        self.identity_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl ServiceProvider<Arc<IdentityProviderRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<IdentityProviderRepository> {
        self.identity_provider_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl ServiceProvider<Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>> for KubernetesBackend {
    fn get(&self) -> Arc<dyn ExternalIdentityValidatorProvider + Send + Sync> {
        self.validator_provider
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl Backend for KubernetesBackend {
    // Nothing here, as this is a marker trait
}

impl IssuerBackend for KubernetesBackend {
    // Nothing here, as this is a marker trait
}

#[async_trait]
impl BackendConfiguration for KubernetesBackend {
    type BackendSettings = BackendSettings;
    type InitializedBackend = KubernetesBackend;

    async fn configure(
        mut self,
        cm: &BackendSettings,
        instance_name: String,
    ) -> anyhow::Result<Arc<Self::InitializedBackend>> {
        info!("Kubernetes backend configuration: {:?}", cm);
        let settings = cm
            .kubernetes
            .as_ref()
            .ok_or(anyhow!("Kubernetes backend configuration is missing"))?;
        let kubeconfig = match settings {
            KubernetesBackendSettings { in_cluster: true, .. } => from_cluster().load()?,
            KubernetesBackendSettings {
                kubeconfig: Some(path), ..
            } => Self::get_from_file(&path).await?,
            KubernetesBackendSettings {
                exec: Some(command), ..
            } => Self::get_from_exec(&command).await?,
            KubernetesBackendSettings {
                kubeconfig: None,
                exec: None,
                ..
            } => {
                bail!("Kubernetes backend configuration is missing")
            }
        };

        let identity_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            (&settings.identity_repository).into(),
        )
        .await?;

        let principal_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            (&settings.principal_repository).into(),
        )
        .await?;

        let schemas_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            (&settings.schema_repository).into(),
        )
        .await?;

        let identity_provider_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            (&settings.identity_provider_repository).into(),
        )
        .await?;

        let validator_provider = KubernetesValidatorProvider::new(identity_provider_repository.clone());

        self.identity_repository = Some(identity_repository);
        self.entities_repository = Some(principal_repository);
        self.schemas_repository = Some(schemas_repository);
        self.identity_provider_repository = Some(identity_provider_repository);
        self.validator_provider = Some(Arc::new(validator_provider));
        info!("Kubernetes backend configured successfully");
        Ok(Arc::new(self))
    }
}

impl KubernetesBackend {
    async fn get_from_exec(command: &str) -> anyhow::Result<Config> {
        info!("Configuring Kubernetes backend with command: {:?}", command);
        let output = Command::new("sh").arg("-c").arg(command).output()?;
        if !output.status.success() {
            bail!(
                "Failed to execute command: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let kubeconfig_string = String::from_utf8(output.stdout)?;
        debug!("Kubeconfig used by the backend:\n{:?}", kubeconfig_string);
        let kubeconfig: Kubeconfig = serde_yml::from_str(&kubeconfig_string)?;
        Ok(Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?)
    }

    async fn get_from_file(path: &str) -> anyhow::Result<Config> {
        info!("Configuring Kubernetes backend with kubeconfig file: {:?}", path);
        let kubeconfig_string = std::fs::read_to_string(path)?;
        debug!("Kubeconfig used by the backend:\n{:?}", kubeconfig_string);
        let kubeconfig: Kubeconfig = serde_yml::from_str(&kubeconfig_string)?;
        Ok(Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?)
    }

    pub async fn create_repository<R>(
        namespace: &str,
        kubeconfig: Config,
        instance_name: String,
        settings: ListenerConfig,
    ) -> anyhow::Result<Arc<KubernetesRepository<R>>>
    where
        R: kube::Resource<Scope = NamespaceResourceScope>
            + SoftDeleteResource
            + UpdateLabels
            + Clone
            + Send
            + Sync
            + 'static,
        R::DynamicType: Hash + Eq + Clone + Default,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            field_manager: instance_name.clone(),
            listener_config: settings,
        };
        KubernetesRepository::<R>::start(config)
            .await
            .map(Arc::new)
            .map_err(|e| e.into())
    }
}
