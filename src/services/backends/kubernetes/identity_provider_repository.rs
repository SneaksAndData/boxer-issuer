pub mod identity_provider_registration;
mod identity_provider_spec;
pub mod oidc_identity_provider_settings;

use crate::services::backends::kubernetes::identity_provider_repository::identity_provider_spec::IdentityProviderDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use identity_provider_registration::IdentityProviderRegistration;

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
