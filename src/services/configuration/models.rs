use crate::services::backends::base::BackendType;
use boxer_core::configuration::models::repository_settings::RepositorySettings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,

    pub identity_repository: RepositorySettings,
    pub principal_repository: RepositorySettings,
    pub schema_repository: RepositorySettings,
    pub identity_provider_repository: RepositorySettings,
}

#[derive(Debug, Deserialize)]
pub struct InitializationSettings {
    pub backend_type: BackendType,
}

#[derive(Debug, Deserialize)]
pub struct BackendSettings {
    pub kubernetes: Option<KubernetesBackendSettings>,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub instance_name: String,
    pub init: InitializationSettings,
    pub backend: BackendSettings,
}
