use crate::services::backends::kubernetes::principal_repository::cedar_entity_document::CedarEntityDocument;
use boxer_core::services::backends::kubernetes::repositories::try_into_object_ref::TryIntoObjectRef;
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

impl TryIntoObjectRef<CedarEntityDocument> for PrincipalIdentity {
    type Error = anyhow::Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<CedarEntityDocument>, Self::Error> {
        let mut components: Vec<&str> = vec![&self.schema_id, self.entity_uid.id().unescaped()];
        components.extend(self.entity_uid.type_name().namespace_components());
        components.join("-").try_into_object_ref(namespace)
    }
}
