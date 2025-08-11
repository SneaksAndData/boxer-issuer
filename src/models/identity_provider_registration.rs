use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IdentityProviderRegistration {
    pub name: String,
    pub oidc: Option<OidcExternalIdentityProviderSettings>,
}
