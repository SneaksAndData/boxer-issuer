use crate::services::external_identity_validator::ExternalIdentityValidator;
use anyhow::bail;
use async_trait::async_trait;
use external_identity::ExternalIdentity;
use jwt_authorizer::Authorizer;
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use token::ExternalToken;

pub mod external_identity;
pub mod token;

pub type DynamicClaimsCollection = HashMap<String, Value>;

pub struct JwtValidator {
    authorizer: Authorizer<DynamicClaimsCollection>,
    user_id_claim: String,
    name: String,
}

#[async_trait]
impl ExternalIdentityValidator for JwtValidator {
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error> {
        let token_str: String = token.into();
        let result = self.authorizer.check_auth(&token_str).await?;
        let maybe_ext_id = extract_user_id(&result.claims, &self.user_id_claim, self.name.clone());
        match maybe_ext_id {
            Some(ext_id) => {
                info!("Successfully validated token for user {}/{}", self.name, ext_id.user_id);
                Ok(ext_id)
            }
            None => bail!("Failed to extract user id from token"),
        }
    }
}

impl JwtValidator {
    pub fn new(authorizer: Authorizer<DynamicClaimsCollection>, user_id_claim: String, name: String) -> JwtValidator {
        JwtValidator {
            authorizer,
            user_id_claim,
            name,
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
