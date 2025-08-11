mod identity_provider_spec;

use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::backends::kubernetes::identity_provider_repository::identity_provider_spec::IdentityProviderDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;

impl UpsertRepositoryWithDelete<String, IdentityProviderRegistration>
    for KubernetesRepository<IdentityProviderDocument>
{
}

pub type IdentityProviderRepository = dyn UpsertRepositoryWithDelete<
    String,
    IdentityProviderRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;
