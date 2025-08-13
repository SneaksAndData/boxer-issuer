mod external_identity;

// Workaround to use prinltn! for logs.
use crate::http::controllers::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::services::backends::kubernetes::identity_repository::external_identity::ExternalIdentityDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{KubernetesRepository, TryIntoObjectRef};
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use kube::runtime::reflector::ObjectRef;

impl TryIntoObjectRef<ExternalIdentityDocument> for (String, String) {
    type Error = anyhow::Error;

    fn try_into_object_ref(self, namespace: String) -> Result<ObjectRef<ExternalIdentityDocument>, Self::Error> {
        format!("{}-{}", self.0, self.1)
            .to_string()
            .try_into_object_ref(namespace)
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
