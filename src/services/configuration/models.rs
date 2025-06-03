use crate::services::backends::base::BackendType;
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InitializationSettings {
    pub backend_type: BackendType,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub init: InitializationSettings,
}

impl AppSettings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(Environment::with_prefix("BOXER").separator("__"))
            .build()?;

        // let hmac = s.clone().try_deserialize::<HashMap<String, String>>()?;
        s.try_deserialize()
    }
}
