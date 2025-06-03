use crate::services::backends::base::BackendType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig_path: String,
}

#[derive(Debug, Deserialize)]
pub struct InitializationSettings {
    pub backend_type: BackendType,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub init: InitializationSettings,
}
