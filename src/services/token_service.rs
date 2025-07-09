use crate::models::api::external::identity::ExternalIdentity;
use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::models::api::internal::v1::token::InternalToken;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::principal_service::PrincipalService;
use async_trait::async_trait;
use cedar_policy::{Entity, SchemaFragment};
use hmac::{Hmac, Mac};
use jwt::{Claims, SignWithKey};
use log::error;
use sha2::Sha256;
use std::sync::Arc;

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
        let schemas = self
            .principal_service
            .get_schemas(principal.get_schema_id().clone())
            .await?;
        self.generate_token(principal.get_entity(), schemas, identity).await
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

    async fn generate_token(
        &self,
        principal: &Entity,
        schemas: SchemaFragment,
        identity: ExternalIdentity,
    ) -> Result<String, anyhow::Error> {
        let token = InternalToken::new(principal.clone(), schemas, identity.user_id, identity.identity_provider);
        let claims: Claims = token.try_into()?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(&self.sign_secret)?;
        claims.sign_with_key(&key).map_err(|e| {
            error!("Failed to issue token: {:?}", e);
            anyhow::anyhow!(e)
        })
    }
}
