use crate::models::external::identity::ExternalIdentity;
use crate::models::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::models::external::token::ExternalToken;
use anyhow::bail;
use async_trait::async_trait;
use jwt_authorizer::error::InitError;
use jwt_authorizer::{Authorizer, AuthorizerBuilder, JwtAuthorizer, Validation};
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Validator for external identity.
#[async_trait]
pub trait ExternalIdentityValidator {
    /// Validate the external identity token and return the external identity.
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error>;
}

/// Instantiates a new external identity validator with given name and settings.
#[async_trait]
pub trait ExternalIdentityValidatorFactory {
    type Error;

    async fn build_validator(
        self,
        name: String,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error>;
}

/// A collection of dynamic claims.
pub type DynamicClaimsCollection = HashMap<String, Value>;

struct ExternalIdentityValidatorImpl {
    authorizer: Authorizer<DynamicClaimsCollection>,
    user_id_claim: String,
    name: String,
}

#[async_trait]
impl ExternalIdentityValidator for ExternalIdentityValidatorImpl {
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error> {
        let token_str: String = token.into();
        let result = self.authorizer.check_auth(&token_str).await?;
        let maybe_ext_id = extract_user_id(&result.claims, &self.user_id_claim, self.name.clone());
        match maybe_ext_id {
            Some(ext_id) => {
                info!(
                    "Successfully validated token for user {}/{}",
                    ext_id.user_id, self.name
                );
                Ok(ext_id)
            }
            None => bail!("Failed to extract user id from token"),
        }
    }
}

fn extract_user_id(
    claims: &DynamicClaimsCollection,
    user_id_claim: &str,
    identity_provider: String,
) -> Option<ExternalIdentity> {
    let value = claims.get(user_id_claim)?;
    let user_id = value.as_str()?.to_owned();
    Some(ExternalIdentity::new(identity_provider, user_id))
}

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
        Ok(Arc::new(ExternalIdentityValidatorImpl {
            authorizer,
            user_id_claim: self.user_id_claim,
            name,
        }))
    }
}
