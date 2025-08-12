use crate::services::backends::kubernetes::principal_repository::cedar_entity_document::CedarEntityDocument;
use boxer_core::services::backends::kubernetes::repositories::IntoObjectRef;
use cedar_policy::EntityUid;
use kube::runtime::reflector::ObjectRef;

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

impl IntoObjectRef<CedarEntityDocument> for PrincipalIdentity {
    fn into_object_ref(self, namespace: String) -> ObjectRef<CedarEntityDocument> {
        let mut components: Vec<&str> = vec![&self.schema_id, self.entity_uid.id().unescaped()];
        components.extend(self.entity_uid.type_name().namespace_components());
        components
            .join("-")
            .to_ascii_lowercase()
            .replace("_", "-")
            .into_object_ref(namespace)
    }
}
