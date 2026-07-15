use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use anyhow::{bail, Error};
use async_trait::async_trait;
use boxer_core::services::external_identity_validator::ExternalIdentityValidator;
use boxer_core::services::external_identity_validator_factory::ExternalIdentityValidatorFactory;
use boxer_core::services::token_service::internal_token_service::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use boxer_core::services::token_service::internal_token_service::external_identity_validator_provider::ExternalIdentityValidatorProvider;
use std::sync::Arc;

pub struct KubernetesValidatorProvider {
    // Add fields as necessary
    repository: Arc<IdentityProviderRepository>,
}

impl KubernetesValidatorProvider {
    pub fn new(repository: Arc<IdentityProviderRepository>) -> Self {
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
        match registration.oidc {
            None => bail!("No OIDC configuration found for provider: {}", provider.name()),
            Some(p) => Ok(p.build_validator(provider.name()).await?),
        }
    }
}
