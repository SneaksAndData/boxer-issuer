use kube::config::Kubeconfig;
use kube::Config;
use log::info;
use std::process::Command;

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
