use crate::services::backends::kubernetes::models::identity_provider::{
    ExternalIdentityInfo, IdentityProvider, IdentityProviderSpec, IdentitySetData,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::config::Kubeconfig;
use kube::{Api, Config};
use log::info;
use maplit::{btreemap, hashset};
use std::process::Command;
use std::sync::Arc;

pub async fn get_kubeconfig() -> anyhow::Result<Config> {
    let output = Command::new("kind")
        .args(&["get", "kubeconfig", "--name", "kind"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get kubeconfig: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let kubeconfig_string = String::from_utf8(output.stdout)?;
    info!("Kubeconfig used by the tests:\n{}", kubeconfig_string);
    let kubeconfig: Kubeconfig = serde_yml::from_str(&kubeconfig_string)?;
    let config = Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?;
    Ok(config)
}

impl ExternalIdentityInfo {
    fn new(name: String) -> Self {
        ExternalIdentityInfo {
            name,
            principal: Default::default(),
        }
    }
}

pub async fn create_mock_identity_providers(
    api: Arc<Api<IdentityProvider>>,
    namespace: String,
    labels_selector_key: &str,
    labels_selector_value: &str,
) {
    let providers = [
        (
            "identity-provider-1",
            hashset!["user1".to_string(), "user2".to_string()],
            hashset![],
        ),
        ("identity-provider-2", hashset!["user1".to_string()], hashset![]),
        (
            "identity-provider-3",
            hashset!["user3".to_string()],
            hashset!["user4".to_string(), "user5".to_string(), "deleted_user".to_string()],
        ),
        ("identity-provider-4", hashset![], hashset![]),
    ];

    for (provider, active, inactive) in &providers {
        let config_map = IdentityProvider {
            spec: IdentityProviderSpec {
                identities: IdentitySetData {
                    active: active
                        .into_iter()
                        .map(|user| ExternalIdentityInfo::new(user.to_string()))
                        .collect(),
                    inactive: inactive
                        .into_iter()
                        .map(|user| ExternalIdentityInfo::new(user.to_string()))
                        .collect(),
                },
                oidc: Default::default(),
            },
            metadata: ObjectMeta {
                name: Some(provider.to_string()),
                namespace: Some(namespace.clone()),
                labels: Some(btreemap! {
                    labels_selector_key.to_string() => labels_selector_value.to_string(),
                    "provider".to_string() => provider.to_string(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        api.create(&PostParams::default(), &config_map)
            .await
            .expect("Failed to create ConfigMap");
    }
}
