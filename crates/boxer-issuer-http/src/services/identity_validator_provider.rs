use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::services::external_identity_validator::ExternalIdentityValidator;
use async_trait::async_trait;
use std::sync::Arc;

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider: Send + Sync {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}
