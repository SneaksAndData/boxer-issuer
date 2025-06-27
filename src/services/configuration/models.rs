use crate::services::backends::base::BackendType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub namespace: String,
    pub label_selector_key: String,
    pub label_selector_value: String,
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
    pub init: InitializationSettings,
    pub backend: BackendSettings,
}
