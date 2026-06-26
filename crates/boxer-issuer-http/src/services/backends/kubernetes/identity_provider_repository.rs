pub mod identity_provider_registration;
mod identity_provider_spec;

use boxer_core::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::GenericKubernetesResourceManager;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use boxer_issuer::services::backends::kubernetes::identity_provider_repository::identity_provider_spec::IdentityProviderDocument;
use identity_provider_registration::IdentityProviderRegistration;

impl UpsertRepositoryWithDelete<String, IdentityProviderRegistration>
    for KubernetesRepository<IdentityProviderDocument, GenericKubernetesResourceManager<IdentityProviderDocument>>
{
}

pub type IdentityProviderRepository = dyn UpsertRepositoryWithDelete<
        String,
        IdentityProviderRegistration,
        DeleteError = Status,
        Error = Status,
        ReadError = Status,
    >;
