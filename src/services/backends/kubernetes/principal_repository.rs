use crate::services::backends::kubernetes::principal_repository::cedar_entity_document::{
    CedarEntityDocument, CedarEntityDocumentSpec,
};
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use boxer_core::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{KubernetesRepository, ToResource, TryFromResource};
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::Entity;
use std::sync::Arc;

mod cedar_entity_document;
pub mod principal_identity;

impl ToResource<CedarEntityDocument> for StoredEntity {
    fn to_resource(&self, object_meta: &kube::api::ObjectMeta) -> Result<CedarEntityDocument, Status> {
        let json_value = self
            .0
            .to_json_value()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        let json_string =
            serde_json::to_string_pretty(&json_value).map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(CedarEntityDocument {
            metadata: object_meta.clone(),
            spec: CedarEntityDocumentSpec {
                active: true,
                entity: json_string,
            },
        })
    }
}

impl TryFromResource<CedarEntityDocument> for StoredEntity {
    type Error = Status;

    fn try_from_resource(resource: Arc<CedarEntityDocument>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Entity::from_json_str(&resource.spec.entity, None)
            .map(StoredEntity)
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl UpsertRepositoryWithDelete<PrincipalIdentity, StoredEntity> for KubernetesRepository<CedarEntityDocument> {}

pub type PrincipalRepository = dyn UpsertRepositoryWithDelete<
    PrincipalIdentity,
    StoredEntity,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

pub struct StoredEntity(Entity);

impl Into<Entity> for StoredEntity {
    fn into(self) -> Entity {
        self.0
    }
}

impl From<Entity> for StoredEntity {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}

impl ToAuditRecord for StoredEntity {
    fn to_audit_record(&self) -> String {
        self.0
            .to_json_string()
            .unwrap_or_else(|_| "<unserializable entity>".to_string())
    }
}
