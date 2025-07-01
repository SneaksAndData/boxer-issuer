mod common;
mod identity_repository;
mod principal_association_repository;
mod principal_repository;
mod schema_repository;

use crate::services::backends::base::{Backend, BackendConfiguration};
use crate::services::backends::kubernetes::common::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::identity_repository::KubernetesIdentityRepository;
use crate::services::backends::kubernetes::principal_association_repository::KubernetesPrincipalAssociationRepository;
use crate::services::backends::kubernetes::principal_repository::KubernetesPrincipalRepository;
use crate::services::backends::kubernetes::schema_repository::KubernetesSchemaRepository;
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository, SchemaRepository,
};
use crate::services::configuration::models::{BackendSettings, KubernetesBackendSettings};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use kube::config::Kubeconfig;
use kube::Config;
use log::{debug, info};
use std::process::Command;
use std::sync::Arc;

pub struct KubernetesBackend {
    pub schemas_repository: Option<Arc<SchemaRepository>>,
    pub entities_repository: Option<Arc<PrincipalRepository>>,
    pub principal_association_repository: Option<Arc<PrincipalAssociationRepository>>,
    pub identity_repository: Option<Arc<IdentityRepository>>,
}

impl KubernetesBackend {
    pub fn new() -> Self {
        KubernetesBackend {
            schemas_repository: None,
            entities_repository: None,
            principal_association_repository: None,
            identity_repository: None,
        }
    }
}

impl Backend for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schemas_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_entities_repository(&self) -> Arc<PrincipalRepository> {
        self.entities_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_principal_association_repository(&self) -> Arc<PrincipalAssociationRepository> {
        self.principal_association_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }

    fn get_identity_repository(&self) -> Arc<IdentityRepository> {
        self.identity_repository
            .as_ref()
            .expect("Backend is not started")
            .clone()
    }
}

#[async_trait]
impl BackendConfiguration for KubernetesBackend {
    async fn configure(mut self, cm: &BackendSettings) -> anyhow::Result<Self> {
        info!("Kubernetes backend configuration: {:?}", cm);
        let settings = cm
            .kubernetes
            .as_ref()
            .ok_or(anyhow!("Kubernetes backend configuration is missing"))?;
        let kubeconfig = match settings {
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
            label_selector_key: settings.label_selector_key.clone(),
            label_selector_value: settings.label_selector_value.clone(),
            kubeconfig,
        };

        let identity_repository = KubernetesIdentityRepository::start(repository_config.clone()).await?;
        let entities_repository = KubernetesPrincipalRepository::start(repository_config.clone()).await?;
        let schemas_repository = KubernetesSchemaRepository::start(repository_config.clone()).await?;
        let principal_association_repository =
            KubernetesPrincipalAssociationRepository::start(repository_config).await?;

        self.schemas_repository = Some(Arc::new(schemas_repository));
        self.entities_repository = Some(Arc::new(entities_repository));
        self.principal_association_repository = Some(Arc::new(principal_association_repository));
        self.identity_repository = Some(Arc::new(identity_repository));
        info!("Kubernetes backend configured successfully");
        Ok(self)
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
