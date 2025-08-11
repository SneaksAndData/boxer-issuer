use crate::services::backends::kubernetes::principal_repository::cedar_entity_document::{
    CedarEntityDocument, CedarEntityDocumentSpec,
};
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{KubernetesRepository, ToResource, TryFromResource};
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::Entity;
use std::sync::Arc;

mod cedar_entity_document;
pub mod principal_identity;

impl ToResource<CedarEntityDocument> for Entity {
    fn to_resource(&self, object_meta: &kube::api::ObjectMeta) -> Result<CedarEntityDocument, Status> {
        let json_string = self
            .to_json_string()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(CedarEntityDocument {
            metadata: object_meta.clone(),
            spec: CedarEntityDocumentSpec {
                active: true,
                entity: json_string,
            },
        })
    }
}

impl TryFromResource<CedarEntityDocument> for Entity {
    type Error = Status;

    fn try_into_resource(resource: Arc<CedarEntityDocument>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Entity::from_json_str(&resource.spec.entity, None).map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl UpsertRepositoryWithDelete<PrincipalIdentity, Entity> for KubernetesRepository<CedarEntityDocument> {}

pub type PrincipalRepository =
    dyn UpsertRepositoryWithDelete<PrincipalIdentity, Entity, DeleteError = Status, Error = Status, ReadError = Status>;
