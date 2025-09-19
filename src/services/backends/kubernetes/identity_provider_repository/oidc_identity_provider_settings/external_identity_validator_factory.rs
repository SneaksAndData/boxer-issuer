use crate::services::backends::kubernetes::identity_provider_repository::oidc_identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::services::external_identity_validator::jwt_validator::{DynamicClaimsCollection, JwtValidator};
use crate::services::external_identity_validator::ExternalIdentityValidator;
use crate::services::external_identity_validator_factory::ExternalIdentityValidatorFactory;
use async_trait::async_trait;
use jwt_authorizer::error::InitError;
use jwt_authorizer::{AuthorizerBuilder, JwtAuthorizer, Validation};
use std::sync::Arc;

#[async_trait]
impl ExternalIdentityValidatorFactory for OidcExternalIdentityProviderSettings {
    type Error = InitError;

    async fn build_validator(
        self,
        name: String,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error> {
        let validation_builder = Validation::new().iss(&self.issuers).aud(&self.audiences);
        let builder: AuthorizerBuilder<DynamicClaimsCollection> =
            JwtAuthorizer::from_oidc(self.discovery_url.as_str()).validation(validation_builder);
        let authorizer = builder.build().await?;
        Ok(Arc::new(JwtValidator::new(authorizer, self.user_id_claim, name)))
    }
}
