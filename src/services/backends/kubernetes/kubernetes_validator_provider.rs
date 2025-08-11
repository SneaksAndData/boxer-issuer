use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use crate::services::external_identity_validator::{ExternalIdentityValidator, ExternalIdentityValidatorFactory};
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use anyhow::{bail, Error};
use async_trait::async_trait;
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
