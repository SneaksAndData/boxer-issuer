pub mod encrypted_token_provider;
pub mod principal;

use crate::services::external_identity_validator::jwt_validator::token::ExternalToken;
use crate::services::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use async_trait::async_trait;

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(
        &self,
        external_identity_provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error>;
}
