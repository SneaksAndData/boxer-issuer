mod external_identity;

use boxer_core::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::GenericKubernetesResourceManager;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use boxer_issuer::http::controllers::v1::identity::external_identity_registration::ExternalIdentityRegistration;
use boxer_issuer::services::backends::kubernetes::identity_repository::external_identity::ExternalIdentityDocument;

impl UpsertRepositoryWithDelete<(String, String), ExternalIdentityRegistration>
    for KubernetesRepository<ExternalIdentityDocument, GenericKubernetesResourceManager<ExternalIdentityDocument>>
{
}

pub type IdentityRepository = dyn UpsertRepositoryWithDelete<
        (String, String),
        ExternalIdentityRegistration,
        DeleteError = Status,
        Error = Status,
        ReadError = Status,
    >;
