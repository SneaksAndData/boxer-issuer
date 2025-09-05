use crate::models::api::external::identity_provider_settings::OidcExternalIdentityProviderSettings;
use boxer_core::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IdentityProviderRegistration {
    pub name: String,
    pub oidc: Option<OidcExternalIdentityProviderSettings>,
}

impl ToAuditRecord for IdentityProviderRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
