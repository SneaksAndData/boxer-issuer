use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::identity_validator_provider::{ExternalIdentityValidationService, ExternalIdentityValidatorProvider};
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::bail;
use log::{error};

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(&self, external_identity_provider: ExternalIdentityProvider, external_token: ExternalToken) -> Result<String, anyhow::Error>;
}

pub  struct TokenService {
    validators: Arc<ExternalIdentityValidationService>
}

#[async_trait]
impl TokenProvider for TokenService
{
    async fn issue_token(&self, provider: ExternalIdentityProvider, external_token: ExternalToken) -> Result<String, anyhow::Error> {
        let validator = self.validators.get(provider.clone()).await?;
        let result = validator.validate(external_token).await;
        match result {
            Ok(identity) => Ok(format!( "successfully logged in as {0} with {1}", identity.user_id, identity.identity_provider).to_string()),
            Err(err) => {
                error!("Failed to validate user token against provider with name {}: {:?}", provider.name(), err);
                bail!("Failed to validate user token against provider with name {}: {:?}", provider.name(), err)
            }
        }
    }
}

impl TokenService {
    pub  fn new(validators: Arc<ExternalIdentityValidationService>) -> Self {
        TokenService {
            validators,
        }
    }
}
