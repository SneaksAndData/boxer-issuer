use std::error::Error;
use async_trait::async_trait;

#[allow(dead_code)]
pub  struct ExternalIdentity{
    pub user_id: String,
    pub identity_provider: String,
}

#[async_trait]
pub trait ExternalIdentityValidator {
    async fn validate(&self, token: &str) -> Result<ExternalIdentity, Box<dyn Error>>;
}

pub struct ExternalIdentityValidatorImpl;

#[async_trait]
impl ExternalIdentityValidator for ExternalIdentityValidatorImpl {
    async fn validate(&self, token: &str) -> Result<ExternalIdentity, Box<dyn Error>> {
        Ok(ExternalIdentity {
            user_id: token.to_string(),
            identity_provider: "google".to_string(),
        })
    }
}
