use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::identity_validator_provider::{
    ExternalIdentityValidationService, ExternalIdentityValidatorProvider,
};
use crate::services::principal_service::PrincipalService;
use async_trait::async_trait;
use cedar_policy::{Entities, SchemaFragment};
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
        let principal = self.principal_service.get_principal(identity.clone()).await?;
        let schemas = self.principal_service.get_schemas(principal.clone()).await?;
        self.generate_token(principal, schemas).await
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

    async fn generate_token(&self, principal: Entities, schemas: SchemaFragment) -> Result<String, anyhow::Error> {
        !todo!()
    }
}
