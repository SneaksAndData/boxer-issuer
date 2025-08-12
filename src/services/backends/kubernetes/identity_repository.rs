mod external_identity;

// Workaround to use prinltn! for logs.
use crate::http::controllers::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::services::backends::kubernetes::identity_repository::external_identity::ExternalIdentityDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{IntoObjectRef, KubernetesRepository};
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use kube::runtime::reflector::ObjectRef;

impl IntoObjectRef<ExternalIdentityDocument> for (String, String) {
    fn into_object_ref(self, namespace: String) -> ObjectRef<ExternalIdentityDocument> {
        let name = format!("{}-{}", self.0, self.1).replace("_", "-");
        let mut or = ObjectRef::new(&name);
        or.namespace = Some(namespace);
        or
    }
}

impl UpsertRepositoryWithDelete<(String, String), ExternalIdentityRegistration>
    for KubernetesRepository<ExternalIdentityDocument>
{
}

pub type IdentityRepository = dyn UpsertRepositoryWithDelete<
    (String, String),
    ExternalIdentityRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;
