use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::services::backends::base::IdentityProviderBackend;
use crate::services::external_identity_validator::{ExternalIdentityValidator, ExternalIdentityValidatorFactory};
use anyhow::bail;
use async_trait::async_trait;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Creates a new external identity validation service.
pub fn new(backend: Arc<dyn IdentityProviderBackend + Send + Sync>) -> ExternalIdentityValidationService {
    ExternalIdentityValidationService::new(backend)
}

/// Write-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorManager {
    async fn put(
        &self,
        provider: ExternalIdentityProvider,
        settings: OidcExternalIdentityProviderSettings,
    ) -> Result<(), anyhow::Error>;
}

#[async_trait]
/// Watcher interface for monitoring changes in external identity providers.
pub trait ExternalIdentityWatcher {
    /// Starts watching for changes in external identity providers.
    async fn watch_for_identity_providers(self);
}

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}

pub struct ExternalIdentityValidationService {
    validators: RwLock<HashMap<ExternalIdentityProvider, Arc<dyn ExternalIdentityValidator + Send + Sync>>>,
    backend: Arc<dyn IdentityProviderBackend + Send + Sync>,
}

#[async_trait]
impl ExternalIdentityValidatorProvider for ExternalIdentityValidationService {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error> {
        let read_guard = self.validators.read().await;
        match (*read_guard).get(&provider) {
            Some(validator) => Ok(Arc::clone(validator)),
            None => bail!("Could not find validator for provider: {}", provider.name()),
        }
    }
}

#[async_trait]
impl ExternalIdentityValidatorManager for ExternalIdentityValidationService {
    async fn put(
        &self,
        provider: ExternalIdentityProvider,
        settings: OidcExternalIdentityProviderSettings,
    ) -> Result<(), anyhow::Error> {
        let mut write_guard = self.validators.write().await;
        let validator = settings.build_validator(provider.name()).await?;
        self.ensure_backend_is_configured(provider.name()).await?;
        let _ = (*write_guard).insert(provider, validator);
        Ok(())
    }
}

impl ExternalIdentityValidationService {
    async fn ensure_backend_is_configured(&self, provider_name: String) -> Result<(), anyhow::Error> {
        self.backend.register_identity_provider(provider_name).await
    }
}

#[async_trait]
impl<T> ExternalIdentityWatcher for Arc<T>
where
    T: ExternalIdentityValidatorManager + Send + Sync,
{
    async fn watch_for_identity_providers(self) {
        let provider = ExternalIdentityProvider::from("provider".to_string());
        let settings = OidcExternalIdentityProviderSettings {
            user_id_claim: "preferred_username".to_string(),
            discovery_url: "http://localhost:8080/realms/master/".to_string(),
            issuers: vec!["http://localhost:8080/realms/master".to_string()],
            audiences: vec!["account".to_string()],
        };
        let result = self.put(provider.clone(), settings).await;
        match result {
            Ok(_) => info!("Successfully updated identity provider settings"),
            Err(e) => error!("Failed to initialize provider with name {}: {:?}", provider.name(), e),
        }
        loop {
            sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}

impl ExternalIdentityValidationService {
    fn new(backend: Arc<dyn IdentityProviderBackend + Send + Sync>) -> Self {
        let validators = RwLock::new(HashMap::new());
        ExternalIdentityValidationService { validators, backend }
    }
}
