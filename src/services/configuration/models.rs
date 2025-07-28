use crate::services::backends::base::BackendType;
use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KubernetesRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
}

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,

    pub lease_name: String,
    pub lease_duration: DurationString,
    pub lease_renew_duration: DurationString,

    pub identity_repository: KubernetesRepositorySettings,
    pub principal_repository: KubernetesRepositorySettings,
    pub schema_repository: KubernetesRepositorySettings,
    pub principal_association_repository: KubernetesRepositorySettings,
    pub identity_provider_repository: KubernetesRepositorySettings,
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
