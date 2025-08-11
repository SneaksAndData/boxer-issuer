use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use boxer_core::services::backends::kubernetes::repositories::{SoftDeleteResource, ToResource, TryFromResource};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "IdentityProviderDocument",
    plural = "identity-providers",
    singular = "identity-provider",
    namespaced
)]
pub struct IdentityProviderDocumentSpec {
    pub oidc: Option<OidcExternalIdentityProviderSettings>,
    pub active: bool,
}

impl UpdateLabels for IdentityProviderDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl Into<IdentityProviderRegistration> for IdentityProviderDocument {
    fn into(self) -> IdentityProviderRegistration {
        IdentityProviderRegistration {
            name: self.metadata.name.unwrap_or_default(),
            oidc: self.spec.oidc,
        }
    }
}

impl Default for IdentityProviderDocument {
    fn default() -> Self {
        IdentityProviderDocument {
            metadata: ObjectMeta::default(),
            spec: IdentityProviderDocumentSpec::default(),
        }
    }
}

impl SoftDeleteResource for IdentityProviderDocument {
    fn is_deleted(&self) -> bool {
        !self.spec.active
    }

    fn set_deleted(&mut self) {
        self.spec.active = false;
    }

    fn clear_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}

impl ToResource<IdentityProviderDocument> for IdentityProviderRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<IdentityProviderDocument, Status> {
        Ok(IdentityProviderDocument {
            metadata: object_meta.clone(),
            spec: IdentityProviderDocumentSpec {
                active: true,
                oidc: self.oidc.clone(),
            },
        })
    }
}

impl TryFromResource<IdentityProviderDocument> for IdentityProviderRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<IdentityProviderDocument>) -> Result<Self, Self::Error> {
        Ok(IdentityProviderRegistration {
            name: resource.metadata.name.clone().unwrap_or_default(),
            oidc: resource.spec.oidc.clone(),
        })
    }
}
