use anyhow::anyhow;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use boxer_core::services::backends::kubernetes::repositories::SoftDeleteResource;
use cedar_policy::Entity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "CedarEntityDocument",
    plural = "cedar-entities",
    singular = "cedar-entity",
    namespaced
)]
pub struct CedarEntityDocumentSpec {
    pub active: bool,
    pub entity: String,
}

impl UpdateLabels for CedarEntityDocument {
    fn update_labels(mut self, custom_labels: &mut std::collections::BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl TryInto<Entity> for CedarEntityDocument {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Entity, Self::Error> {
        Entity::from_json_str(self.spec.entity, None).map_err(|err| anyhow!(err))
    }
}

impl Default for CedarEntityDocument {
    fn default() -> Self {
        CedarEntityDocument {
            metadata: ObjectMeta::default(),
            spec: CedarEntityDocumentSpec::default(),
        }
    }
}

impl SoftDeleteResource for CedarEntityDocument {
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
