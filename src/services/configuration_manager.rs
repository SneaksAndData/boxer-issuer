use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use crate::services::identity_validator_provider::ExternalIdentityValidatorManager;
use async_trait::async_trait;
use std::sync::Arc;
use log::{error, info};
use tokio::time::sleep;

#[async_trait]
/// A trait for managing application configuration updates.
pub trait ConfigurationManager {
    
    /// Watches for IdentityProviderSettings update and upserts them into the identity validator provider.
   async fn watch_for_identity_providers(self);
}

/// Dummy implementation of the ConfigurationManager trait.
#[async_trait]
impl<T> ConfigurationManager for Arc<T> where T: ExternalIdentityValidatorManager + Send + Sync {
    async fn watch_for_identity_providers(self)
    {
        let provider = ExternalIdentityProvider::from("provider".to_string());
        let settings = OidcExternalIdentityProviderSettings {
            user_id_claim: "upn".to_string(),
            discovery_url: "https://example.com/".to_string(),
            issuers: vec!("https://example.com/".to_string()),
            audiences: vec!("https://example.com/".to_string())
        };
        let result = self.put(provider.clone(), settings).await;
        match result {
            Ok(_) => info!("Successfully updated identity provider settings"),
            Err(e) => error!("Failed to initialize provider with name {}: {:?}", provider.name(), e)
        }
        loop {
            sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}
