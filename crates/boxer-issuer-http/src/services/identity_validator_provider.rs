use async_trait::async_trait;
use boxer_issuer::models::api::external::identity_provider::ExternalIdentityProvider;
use boxer_issuer::services::external_identity_validator::ExternalIdentityValidator;
use std::sync::Arc;

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider: Send + Sync {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}
