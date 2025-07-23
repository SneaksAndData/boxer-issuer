use crate::services::backends::base::BackendType;
use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PrincipalAssociationRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
}

#[derive(Debug, Deserialize)]
pub struct PrincipalRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
}

#[derive(Debug, Deserialize)]
pub struct SchemaRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
}

#[derive(Debug, Deserialize)]
pub struct IdentityRepositorySettings {
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

    pub identity_repository: IdentityRepositorySettings,
    pub principal_repository: PrincipalRepositorySettings,
    pub schema_repository: SchemaRepositorySettings,
    pub principal_association_repository: PrincipalAssociationRepositorySettings,
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
