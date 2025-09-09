use crate::services::backends::base::BackendType;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
/// Trait managing the initialization configuration for the application.
pub trait InitializationConfigurationManager {
    /// Retrieves the signing key used for token generation.
    fn get_signing_key(&self) -> Arc<Vec<u8>>;

    /// Retrieves the signing key used for token generation.
    fn get_key_id(&self) -> String;

    /// Retrieves the backend type for the application.
    fn get_backend_type(&self) -> BackendType;

    /// Retrieves the audience for the tokens.
    fn get_audience(&self) -> String;

    /// Retrieves the issuer for the tokens.
    fn get_issuer(&self) -> String;

    /// Retrieves the content encryption algorithm for the tokens.
    fn get_content_encryption(&self) -> String;
}
