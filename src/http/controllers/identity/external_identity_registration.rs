use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
/// Struct that represents an external identity
pub struct ExternalIdentityRegistration {
    /// The user ID extracted from the external identity provider
    pub user_id: String,

    /// The name of the external identity provider
    pub identity_provider: String,

    /// The principal ID associated with the external identity
    pub principal_id: String,

    /// The schema of the principal associated with the external identity
    pub principal_schema: String,
}
