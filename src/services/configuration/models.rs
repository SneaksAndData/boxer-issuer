use crate::services::backends::base::BackendType;
use boxer_core::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use duration_string::DurationString;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,
    pub operation_timeout: DurationString,
    pub resource_owner_label: String,
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
    pub listen_address: SocketAddr,
    pub init: InitializationSettings,
    pub backend: BackendSettings,
    pub opentelemetry: OpenTelemetrySettings,
}
