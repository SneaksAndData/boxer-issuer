pub mod common;
mod identity_provider_repository;
mod identity_repository;
pub mod models;
mod principal_association_repository;
mod principal_repository;

mod kubernetes_validator_provider;

use crate::services::backends::base::{
    EntitiesRepositorySource, ExternalIdentityValidatorProviderSource, IdentityProviderRepositorySource,
    IdentityRepositorySource, IssuerBackend, PrincipalAssociationRepositorySource,
};
use crate::services::backends::kubernetes::identity_provider_repository::KubernetesIdentityProviderRepository;
use crate::services::backends::kubernetes::identity_repository::KubernetesIdentityRepository;
use crate::services::backends::kubernetes::principal_association_repository::KubernetesPrincipalAssociationRepository;
use crate::services::backends::kubernetes::principal_repository::KubernetesPrincipalRepository;
use crate::services::base::upsert_repository::{
    IdentityProviderRepository, IdentityRepository, PrincipalAssociationRepository, PrincipalRepository,
};
use crate::services::configuration::models::{BackendSettings, KubernetesBackendSettings};
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::from_cluster;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::KubernetesSchemaRepository;
use boxer_core::services::backends::{Backend, BackendConfiguration, SchemaRepositorySource};
use boxer_core::services::base::types::SchemaRepository;
use kube::config::Kubeconfig;
use kube::Config;
use kubernetes_validator_provider::KubernetesValidatorProvider;
use log::{debug, info};
use std::process::Command;
use std::sync::Arc;

pub struct KubernetesBackend {
    pub schemas_repository: Option<Arc<SchemaRepository>>,
    pub entities_repository: Option<Arc<PrincipalRepository>>,
    pub principal_association_repository: Option<Arc<PrincipalAssociationRepository>>,
    pub identity_repository: Option<Arc<KubernetesIdentityRepository>>,
    pub identity_provider_repository: Option<Arc<KubernetesIdentityProviderRepository>>,
    pub validator_provider: Option<Arc<KubernetesValidatorProvider>>,
}

impl KubernetesBackend {
    pub fn new() -> Self {
        KubernetesBackend {
            schemas_repository: None,
            entities_repository: None,
            principal_association_repository: None,
            identity_repository: None,
            identity_provider_repository: None,
            validator_provider: None,
        }
    }
}

impl SchemaRepositorySource for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schemas_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl EntitiesRepositorySource for KubernetesBackend {
    fn get_entities_repository(&self) -> Arc<PrincipalRepository> {
        self.entities_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl PrincipalAssociationRepositorySource for KubernetesBackend {
    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository> {
        self.principal_association_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl ExternalIdentityValidatorProviderSource for KubernetesBackend {
    fn get_external_identity_validator_provider(&self) -> Arc<dyn ExternalIdentityValidatorProvider> {
        self.validator_provider
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

impl IdentityProviderRepositorySource for KubernetesBackend {
    fn get_identity_provider_repository(&self) -> Arc<IdentityProviderRepository> {
        self.identity_provider_repository
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

impl IdentityRepositorySource for KubernetesBackend {
    fn get_identity_repository(&self) -> Arc<IdentityRepository> {
        self.identity_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
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

        let repository_config = KubernetesResourceManagerConfig {
            namespace: settings.namespace.clone(),
            label_selector_key: settings.identity_repository.label_selector_key.clone(),
            label_selector_value: settings.identity_repository.label_selector_value.clone(),
            lease_name: settings.lease_name.clone(),
            lease_duration: settings.lease_duration.into(),
            renew_deadline: settings.lease_renew_duration.into(),
            claimant: instance_name,
            kubeconfig,
        };

        let identity_repository = KubernetesIdentityRepository::start(repository_config.clone()).await?;
        let entities_repository = KubernetesPrincipalRepository::start(repository_config.clone_with_label_selector(
            settings.principal_repository.label_selector_key.clone(),
            settings.principal_repository.label_selector_value.clone(),
        ))
        .await?;

        let schemas_repository = KubernetesSchemaRepository::start(repository_config.clone_with_label_selector(
            settings.schema_repository.label_selector_key.clone(),
            settings.schema_repository.label_selector_value.clone(),
        ))
        .await?;

        let principal_association_repository =
            KubernetesPrincipalAssociationRepository::start(repository_config.clone_with_label_selector(
                settings.principal_association_repository.label_selector_key.clone(),
                settings.principal_association_repository.label_selector_value.clone(),
            ))
            .await?;

        let identity_provider_repository =
            KubernetesIdentityProviderRepository::start(repository_config.clone_with_label_selector(
                settings.identity_provider_repository.label_selector_key.clone(),
                settings.identity_provider_repository.label_selector_value.clone(),
            ))
            .await?;

        let identity_provider_repository = Arc::new(identity_provider_repository);

        let validator_provider = KubernetesValidatorProvider::new(identity_provider_repository.clone());

        self.schemas_repository = Some(Arc::new(schemas_repository));
        self.entities_repository = Some(Arc::new(entities_repository));
        self.principal_association_repository = Some(Arc::new(principal_association_repository));
        self.identity_repository = Some(Arc::new(identity_repository));
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
}
