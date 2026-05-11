use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
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
use boxer_core::services::audit::audit_facade::WithAuditFacade;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::{from_cluster, from_command, from_file};
use boxer_core::services::backends::kubernetes::kubernetes_repository::schema_repository::SchemaRepository;
use boxer_core::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use boxer_core::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    GenericKubernetesResourceManager, KubernetesResourceManagerConfig, UpdateLabels,
};
use boxer_core::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use boxer_core::services::backends::{Backend, BackendConfiguration};
use boxer_core::services::service_provider::ServiceProvider;
use k8s_openapi::NamespaceResourceScope;
use kube::Config;
use kubernetes_validator_provider::KubernetesValidatorProvider;
use log::info;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct KubernetesBackend {
    pub schemas_repository: Option<Arc<SchemaRepository>>,
    pub entities_repository: Option<Arc<PrincipalRepository>>,
    pub identity_repository: Option<Arc<IdentityRepository>>,
    pub identity_provider_repository: Option<Arc<IdentityProviderRepository>>,
    pub validator_provider: Option<Arc<KubernetesValidatorProvider>>,
    readiness_state: Arc<AtomicBool>,
}

impl KubernetesBackend {
    pub fn new() -> Self {
        KubernetesBackend {
            schemas_repository: None,
            entities_repository: None,
            identity_repository: None,
            identity_provider_repository: None,
            validator_provider: None,
            readiness_state: Arc::new(AtomicBool::new(false)),
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
    fn readiness_state(&self) -> Arc<AtomicBool> {
        self.readiness_state.clone()
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
            } => from_file().load(&path).await?,
            KubernetesBackendSettings {
                exec: Some(command), ..
            } => from_command().load(&command).await?,
            KubernetesBackendSettings {
                kubeconfig: None,
                exec: None,
                ..
            } => {
                bail!("Kubernetes backend configuration is missing")
            }
        };

        let owner_mark = ObjectOwnerMark::new(&settings.resource_owner_label, &instance_name);

        let (identity_repository, identity_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let identity_repository = identity_repository.with_audit(Arc::new(LogAuditService::new()));

        let (principal_repository, principal_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let principal_repository = principal_repository.with_audit(Arc::new(LogAuditService::new()));

        let (schemas_repository, schemas_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let schemas_repository = schemas_repository.with_audit(Arc::new(LogAuditService::new()));

        let (identity_provider_repository, identity_provider_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let identity_provider_repository = identity_provider_repository.with_audit(Arc::new(LogAuditService::new()));

        let validator_provider = KubernetesValidatorProvider::new(identity_provider_repository.clone());

        self.identity_repository = Some(identity_repository);
        self.entities_repository = Some(principal_repository);
        self.schemas_repository = Some(schemas_repository);
        self.identity_provider_repository = Some(identity_provider_repository);
        self.validator_provider = Some(Arc::new(validator_provider));
        let readiness_state = self.readiness_state.clone();
        tokio::spawn(async move {
            let is_ready = identity_repository_readiness.await.is_ok()
                && principal_repository_readiness.await.is_ok()
                && schemas_repository_readiness.await.is_ok()
                && identity_provider_repository_readiness.await.is_ok();
            readiness_state.store(is_ready, Ordering::Release);
        });
        info!("Kubernetes backend configured successfully");
        Ok(Arc::new(self))
    }
}

impl KubernetesBackend {
    pub async fn create_repository<R>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<(
        Arc<KubernetesRepository<R, GenericKubernetesResourceManager<R>>>,
        tokio::sync::oneshot::Receiver<()>,
    )>
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
            owner_mark,
            operation_timeout,
        };
        let (resource_manager, readiness_rx) =
            GenericKubernetesResourceManager::start(config, Arc::new(LoggingUpdateHandler)).await?;
        let repository =
            KubernetesRepository::<R, GenericKubernetesResourceManager<R>>::start(resource_manager, operation_timeout)
                .await
                .map(Arc::new)?;
        Ok((repository, readiness_rx))
    }
}
