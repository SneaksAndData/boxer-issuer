use crate::models::external::identity::{ExternalIdentity, Policy};
use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::models::internal::v1::token::InternalToken;
use crate::services::identity_validator_provider::{
    ExternalIdentityValidationService, ExternalIdentityValidatorProvider,
};
use anyhow::bail;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use jwt::{Claims, SignWithKey};
use log::error;
use sha2::Sha256;
use std::sync::Arc;
use crate::services::base::upsert_repository::{PolicyAttachmentRepository, PolicyRepository};

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(
        &self,
        external_identity_provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error>;
    async fn generate_token(&self, identity: ExternalIdentity) -> Result<String, anyhow::Error>;
}

pub struct TokenService {
    validators: Arc<ExternalIdentityValidationService>,
    policy_attachment_repository: Arc<PolicyAttachmentRepository>,
    policy_repository: Arc<PolicyRepository>,
    sign_secret: Arc<Vec<u8>>,
}

#[async_trait]
impl TokenProvider for TokenService {
    async fn issue_token(
        &self,
        provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error> {
        let validator = self.validators.get(provider.clone()).await?;
        let result = validator.validate(external_token).await;
        match result {
            Ok(identity) => self.generate_token(identity).await,
            Err(err) => {
                error!(
                    "Failed to validate user token against provider with name {}: {:?}",
                    provider.name(),
                    err
                );
                bail!(
                    "Failed to validate user token against provider with name {}: {:?}",
                    provider.name(),
                    err
                )
            }
        }
    }
    async fn generate_token(&self, identity: ExternalIdentity) -> Result<String, anyhow::Error> {
        let attachment= self.policy_attachment_repository.get(identity.clone()).await?;
        let policies = Policy::empty();
        for p in attachment.policies {
            let policy = self.policy_repository.get(p).await?;
            policies.merge(policy);
        }
        
        let token = InternalToken::new(policies, identity.user_id, identity.identity_provider);
        let claims: Claims = token.try_into()?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(&self.sign_secret)?;
        claims.sign_with_key(&key).map_err(|e| {
            error!("Failed to issue token: {:?}", e);
            anyhow::anyhow!(e)
        })
    }
}

impl TokenService {
    pub fn new(
        validators: Arc<ExternalIdentityValidationService>,
        policy_repository: Arc<PolicyRepository>,
        policy_attachment_repository: Arc<PolicyAttachmentRepository>,
        sign_secret: Arc<Vec<u8>>,
    ) -> Self {
        TokenService {
            validators,
            policy_repository,
            policy_attachment_repository,
            sign_secret,
        }
    }
}
