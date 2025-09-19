use crate::services::external_identity_validator::ExternalIdentityValidator;
use async_trait::async_trait;
use std::sync::Arc;

/// Instantiates a new external identity validator with given name and settings.
#[async_trait]
pub trait ExternalIdentityValidatorFactory {
    type Error;

    async fn build_validator(
        self,
        name: String,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error>;
}
