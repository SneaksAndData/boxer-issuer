use crate::models::external::token::ExternalToken;
use async_trait::async_trait;
use jwt_authorizer::error::InitError;
use jwt_authorizer::{Authorizer, AuthorizerBuilder, JwtAuthorizer, Validation};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use log::info;
use serde_json::Value;
use crate::models::external::identity::ExternalIdentity;
use crate::models::external::identity_provider_settings::OidcExternalIdentityProviderSettings;

/// Validator for external identity.
#[async_trait]
pub trait ExternalIdentityValidator {
    
    /// Validate the external identity token and return the external identity.
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, Box<dyn Error>>;
}

/// Instantiates a new external identity validator with given name and settings.
#[async_trait]
pub trait ExternalIdentityValidatorFactory {
    type Error;
    
    async fn build_validator(self, name: String) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error>;
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
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, Box<dyn Error>> {
        let token_str: String = token.into();
        let result = self.authorizer.check_auth(&token_str).await;
        
        match result {
            Ok(data) => extract_user_id(&data.claims, &self.user_id_claim).map(|user_id| ExternalIdentity {
                user_id,
                identity_provider: self.name.clone(),
            }),
            Err(_) => {
                info!("Provided token is invalid");
                Err("Unauthorized".into())
            }
        }
    }
}

fn extract_user_id(claims: &DynamicClaimsCollection, user_id_claim: &str) -> Result<String, Box<dyn Error>> {
    let maybe_user_id = claims.get(user_id_claim);
    match maybe_user_id {
        Some(user) => Ok(user.to_owned().to_string()),
        None => {
            info!("User ID claim '{}' not found in the token", user_id_claim);
            Err("Unauthorized".into())
        }
    }
}

#[async_trait]
impl ExternalIdentityValidatorFactory for OidcExternalIdentityProviderSettings {
    type Error = InitError;
    
    async fn build_validator(self, name: String) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error> {
        let validation_builder= Validation::new().iss(&self.issuers).aud(&self.audiences);
        let builder: AuthorizerBuilder<DynamicClaimsCollection> = JwtAuthorizer::from_oidc(self.discovery_url.as_str()).validation(validation_builder);
        let authorizer = builder.build().await?;
        Ok(Arc::new(ExternalIdentityValidatorImpl {
            authorizer,
            user_id_claim: self.user_id_claim,
            name
        }))
    }
}
