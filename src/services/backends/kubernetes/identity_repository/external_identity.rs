use crate::http::controllers::v1::identity::external_identity_registration::ExternalIdentityRegistration;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use boxer_core::services::backends::kubernetes::repositories::{SoftDeleteResource, ToResource, TryFromResource};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct PrincipalReference {
    pub principal: String,
    pub schema: String,
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "ExternalIdentityDocument",
    plural = "external-identities",
    singular = "external-identity",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct ExternalIdentityDocumentSpec {
    pub active: bool,
    pub id: String,
    pub identity_provider: String,
    pub principal_ref: PrincipalReference,
    pub validator_schema_id: String,
}

impl UpdateLabels for ExternalIdentityDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl Into<ExternalIdentityRegistration> for ExternalIdentityDocumentSpec {
    fn into(self) -> ExternalIdentityRegistration {
        ExternalIdentityRegistration {
            id: self.id,
            identity_provider: self.identity_provider,
            principal_id: self.principal_ref.principal,
            principal_schema: self.principal_ref.schema,
            validator_schema: self.validator_schema_id,
        }
    }
}

impl Default for ExternalIdentityDocument {
    fn default() -> Self {
        ExternalIdentityDocument {
            metadata: kube::api::ObjectMeta {
                name: None,
                namespace: None,
                labels: Some(BTreeMap::new()),
                ..Default::default()
            },
            spec: ExternalIdentityDocumentSpec::default(),
        }
    }
}

impl SoftDeleteResource for ExternalIdentityDocument {
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

impl ToResource<ExternalIdentityDocument> for ExternalIdentityRegistration {
    fn to_resource(&self, object_meta: &kube::api::ObjectMeta) -> Result<ExternalIdentityDocument, Status> {
        Ok(ExternalIdentityDocument {
            metadata: object_meta.clone(),
            spec: ExternalIdentityDocumentSpec {
                active: true,
                id: self.id.clone(),
                identity_provider: self.identity_provider.clone(),
                principal_ref: PrincipalReference {
                    principal: self.principal_id.clone(),
                    schema: self.principal_schema.clone(),
                },
                validator_schema_id: self.validator_schema.clone(),
            },
        })
    }
}

impl TryFromResource<ExternalIdentityDocument> for ExternalIdentityRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<ExternalIdentityDocument>) -> Result<Self, Self::Error> {
        Ok(ExternalIdentityRegistration {
            id: resource.spec.id.clone(),
            identity_provider: resource.spec.identity_provider.clone(),
            principal_id: resource.spec.principal_ref.principal.clone(),
            principal_schema: resource.spec.principal_ref.schema.clone(),
            validator_schema: resource.spec.validator_schema_id.clone(),
        })
    }
}
