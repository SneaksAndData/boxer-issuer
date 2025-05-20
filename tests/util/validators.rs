use anyhow::Error;
use async_trait::async_trait;
use boxer_issuer::models::api::external::identity::ExternalIdentity;
use boxer_issuer::models::api::external::identity_provider::ExternalIdentityProvider;
use boxer_issuer::models::api::external::token::ExternalToken;
use boxer_issuer::services::external_identity_validator::ExternalIdentityValidator;
use boxer_issuer::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AlwaysValid {
    pub identity: ExternalIdentity,
}

#[async_trait]
impl ExternalIdentityValidator for AlwaysValid {
    async fn validate(&self, _: ExternalToken) -> Result<ExternalIdentity, Error> {
        Ok(self.identity.clone())
    }
}

#[async_trait]
impl ExternalIdentityValidatorProvider for AlwaysValid {
    async fn get(
        &self,
        _: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Error> {
        Ok(Arc::new(self.clone()))
    }
}
