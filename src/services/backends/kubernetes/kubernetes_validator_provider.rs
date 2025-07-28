use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::identity_provider_registration::IdentityProviderRegistration;
use crate::services::external_identity_validator::{ExternalIdentityValidator, ExternalIdentityValidatorFactory};
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use anyhow::Error;
use async_trait::async_trait;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use std::sync::Arc;

pub struct KubernetesValidatorProvider {
    // Add fields as necessary
    repository: Arc<dyn ReadOnlyRepository<String, IdentityProviderRegistration, ReadError = Error>>,
}

impl KubernetesValidatorProvider {
    pub fn new(
        repository: Arc<dyn ReadOnlyRepository<String, IdentityProviderRegistration, ReadError = Error>>,
    ) -> Self {
        KubernetesValidatorProvider { repository }
    }
}

#[async_trait]
impl ExternalIdentityValidatorProvider for KubernetesValidatorProvider {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Error> {
        let registration = self.repository.get(provider.name()).await?;
        Ok(registration.oidc.build_validator(provider.name()).await?)
    }
}
