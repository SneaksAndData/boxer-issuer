use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
/// Struct that represents an external identity
pub struct ExternalIdentityRegistrationRequest {
    /// The principal ID associated with the external identity
    pub principal_id: String,

    /// The schema of the principal associated with the external identity
    pub principal_schema: String,

    /// The schema ID used fot token validation
    pub validator_schema: String,
}
