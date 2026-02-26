use crate::services::backends::kubernetes::principal_repository::cedar_entity_document::CedarEntityDocument;
use boxer_core::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use boxer_core::services::backends::kubernetes::kubernetes_repository::try_into_object_ref::TryIntoObjectRef;
use cedar_policy::EntityUid;
use kube::runtime::reflector::ObjectRef;
use serde::ser::SerializeStruct;
use serde::Serialize;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PrincipalIdentity {
    schema_id: String,
    entity_uid: EntityUid,
}

impl PrincipalIdentity {
    pub fn new(schema_id: String, entity_uid: EntityUid) -> Self {
        Self { schema_id, entity_uid }
    }
}

impl Serialize for PrincipalIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PrincipalIdentity", 2)?;
        state.serialize_field("schemaId", &self.schema_id)?;
        state.serialize_field("entityUid", &self.entity_uid.to_string())?;
        state.end()
    }
}

impl TryIntoObjectRef<CedarEntityDocument> for PrincipalIdentity {
    type Error = anyhow::Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<CedarEntityDocument>, Self::Error> {
        let mut components: Vec<&str> = vec![&self.schema_id, self.entity_uid.id().unescaped()];
        components.extend(self.entity_uid.type_name().namespace_components());
        components.join("-").try_into_object_ref(namespace)
    }
}

impl ToAuditRecord for PrincipalIdentity {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
