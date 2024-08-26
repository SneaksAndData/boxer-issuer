use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::services::external_identity_validator::{ExternalIdentityValidator, ExternalIdentityValidatorFactory};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::bail;
use tokio::sync::RwLock;

/// Creates a new external identity validation service.
pub fn new() -> ExternalIdentityValidationService {
    ExternalIdentityValidationService::new()
}

/// Write-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorManager {
    async fn put(&self, provider: ExternalIdentityProvider, settings: OidcExternalIdentityProviderSettings) -> Result<(), anyhow::Error>;
}

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider {
    async fn get(&self, provider: ExternalIdentityProvider) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}


pub struct ExternalIdentityValidationService {
    validators: RwLock<HashMap<ExternalIdentityProvider, Arc<dyn ExternalIdentityValidator + Send + Sync>>>,
}

#[async_trait]
impl ExternalIdentityValidatorProvider for ExternalIdentityValidationService {
    async fn get(&self, provider: ExternalIdentityProvider) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error> {
        let read_guard = self.validators.read().await;
        match (*read_guard).get(&provider){
            Some(validator) => Ok(Arc::clone(validator)),
            None => bail!("Could not find validator for provider: {}", provider.name()),
        }
    }
}

#[async_trait]
impl ExternalIdentityValidatorManager for ExternalIdentityValidationService {
    async fn put(&self, provider: ExternalIdentityProvider, settings: OidcExternalIdentityProviderSettings) -> Result<(), anyhow::Error> {
        let mut write_guard = self.validators.write().await;
        let validator = settings.build_validator(provider.name()).await?;
        let _ = (*write_guard).insert(provider,  validator);
        Ok(())
    }
}

impl ExternalIdentityValidationService {
    fn new() -> Self {
        let validators = RwLock::new(HashMap::new());
        ExternalIdentityValidationService {
            validators,
        }
    }
}
