pub mod jwt_validator;

use async_trait::async_trait;
use jwt_validator::external_identity::ExternalIdentity;
use jwt_validator::token::ExternalToken;

/// Validator for external identity.
#[async_trait]
pub trait ExternalIdentityValidator {
    /// Validate the external identity token and return the external identity.
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error>;
}
