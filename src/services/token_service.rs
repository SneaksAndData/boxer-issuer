use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::principal_service::PrincipalService;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::TokenBuilder;
use hmac::{Hmac, Mac};
use jwt::{Claims, SignWithKey};
use log::error;
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(
        &self,
        external_identity_provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error>;
}

pub struct TokenService {
    validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
    principal_service: Arc<PrincipalService>,
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
        let identity = validator.validate(external_token).await?;
        let principal = self.principal_service.get_principal(identity.clone()).await?;
        let schema_name = principal.get_schema_id().clone();
        let schemas = self.principal_service.get_schemas(schema_name.clone()).await?;
        let validator_schema_id = self.principal_service.get_validator_schema(identity.clone()).await?;
        let token = TokenBuilder::new()
            .principal(principal.get_entity().clone())
            .schema(schemas)
            .user_id(identity.user_id)
            .identity_provider(identity.identity_provider)
            .schema_name(schema_name)
            .validity_period(Duration::from_secs(3600))
            .validator_schema_id(validator_schema_id)
            .build()?;
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
        validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
        principal_service: Arc<PrincipalService>,
        sign_secret: Arc<Vec<u8>>,
    ) -> Self {
        TokenService {
            validators,
            principal_service,
            sign_secret,
        }
    }
}
