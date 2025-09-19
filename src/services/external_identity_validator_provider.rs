use crate::services::external_identity_validator::ExternalIdentityValidator;
use async_trait::async_trait;
use external_identity_provider::ExternalIdentityProvider;
use std::sync::Arc;

/// This module contains all the models used in the application.
pub mod external_identity_provider;

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider: Send + Sync {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}
