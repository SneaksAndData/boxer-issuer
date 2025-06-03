use crate::services::backends::base::BackendType;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
/// A trait for managing application configuration updates.
pub trait InitializationConfigurationManager {
    /// Reads the key for signing the issued tokens
    fn get_signing_key(&self) -> Arc<Vec<u8>>;

    fn get_backend_type(&self) -> BackendType;
}
