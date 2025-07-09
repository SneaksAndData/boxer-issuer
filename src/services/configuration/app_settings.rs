use crate::services::backends::base::BackendType;
use crate::services::configuration::base::initialization_configuration_manager::InitializationConfigurationManager;
use crate::services::configuration::models::AppSettings;
use async_trait::async_trait;
use config::{Config, ConfigError, Environment, File};
use std::sync::Arc;

impl AppSettings {
    /// Creates a new instance of `AppSettings` by loading configuration from predefined sources
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("settings.toml"))
            .add_source(Environment::with_prefix("BOXER").separator("__"))
            .build()?;

        // let hmac = s.clone().try_deserialize::<HashMap<String, String>>()?;
        s.try_deserialize()
    }
}

/// Dummy implementation of the InitializationConfigurationManager trait.
#[async_trait]
impl InitializationConfigurationManager for AppSettings {
    fn get_signing_key(&self) -> Arc<Vec<u8>> {
        Arc::new(vec!["dummy-secret".as_bytes()].concat())
    }

    fn get_backend_type(&self) -> BackendType {
        self.init.backend_type.clone()
    }
}
