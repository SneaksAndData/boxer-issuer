use crate::services::backends::base::BackendType;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
/// Trait managing the initialization configuration for the application.
pub trait InitializationConfigurationManager {
    /// Retrieves the signing key used for token generation.
    fn get_signing_key(&self) -> Arc<Vec<u8>>;

    /// Retrieves the backend type for the application.
    fn get_backend_type(&self) -> BackendType;
}
