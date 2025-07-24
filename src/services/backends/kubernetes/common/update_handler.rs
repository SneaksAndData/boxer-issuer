use crate::services::backends::kubernetes::common::ResourceUpdateHandler;
use crate::services::backends::kubernetes::models::identity_provider::IdentityProvider;
use futures::future;
use futures::future::Ready;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::watcher;
use log::{debug, warn};

pub struct UpdateHandler;
impl ResourceUpdateHandler<IdentityProvider> for UpdateHandler {
    fn handle_update(&self, event: Result<IdentityProvider, watcher::Error>) -> Ready<()> {
        match event {
            Ok(IdentityProvider {
                metadata:
                    ObjectMeta {
                        name: Some(name),
                        namespace: Some(namespace),
                        ..
                    },
                spec: _,
            }) => debug!("Saw [{}] in [{}]", name, namespace),
            Ok(_) => warn!("Saw an object without name or namespace"),
            Err(e) => warn!("watcher error: {}", e),
        }
        future::ready(())
    }
}
