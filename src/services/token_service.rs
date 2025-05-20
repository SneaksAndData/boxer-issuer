use crate::models::api::external::identity::{ExternalIdentity, Policy};
use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::models::api::internal::v1::token::InternalToken;
use crate::services::base::upsert_repository::{PolicyAttachmentRepository, PolicyRepository};
use crate::services::identity_validator_provider::{
    ExternalIdentityValidationService, ExternalIdentityValidatorProvider,
};
use crate::services::principal_service::PrincipalService;
use async_trait::async_trait;
use cedar_policy::{Entities, SchemaFragment};
use std::sync::Arc;
use hmac::{Hmac, Mac};
use jwt::{Claims, SignWithKey};
use log::error;
use sha2::Sha256;
use crate::models::external::identity::ExternalIdentity;
use crate::models::internal::v1::token::InternalToken;

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(
        &self,
        external_identity_provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error>;
}

pub struct TokenService {
    validators: Arc<ExternalIdentityValidationService>,
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
        let (principal, schema_id) = self.principal_service.get_principal(identity.clone()).await?;
        let schemas = self.principal_service.get_schemas(schema_id).await?;
        self.generate_token(principal, schemas, identity).await
    }
}

impl TokenService {
    pub fn new(
        validators: Arc<ExternalIdentityValidationService>,
        principal_service: Arc<PrincipalService>,
        sign_secret: Arc<Vec<u8>>,
    ) -> Self {
        TokenService {
            validators,
            principal_service,
            sign_secret,
        }
    }

    async fn generate_token(&self, principal: Entities, schemas: SchemaFragment, identity: ExternalIdentity) -> Result<String, anyhow::Error> {
        let token = InternalToken::new(principal, schemas, identity.user_id, identity.identity_provider);
        let claims: Claims = token.try_into()?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(&self.sign_secret)?;
        claims.sign_with_key(&key).map_err(|e| {
            error!("Failed to issue token: {:?}", e);
            anyhow::anyhow!(e)
        })
    }
}
