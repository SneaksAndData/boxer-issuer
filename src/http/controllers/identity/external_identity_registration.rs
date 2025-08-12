use crate::http::controllers::identity::external_identity_registration_request::ExternalIdentityRegistrationRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
/// Struct that represents an external identity
pub struct ExternalIdentityRegistration {
    /// The user ID extracted from the external identity provider
    pub id: String,

    /// The name of the external identity provider
    pub identity_provider: String,

    /// The principal ID associated with the external identity
    pub principal_id: String,

    /// The schema of the principal associated with the external identity
    pub principal_schema: String,
}

impl ExternalIdentityRegistration {
    /// Creates a new instance of `ExternalIdentityRegistration`
    pub fn from_request(identity_provider: String, id: String, request: ExternalIdentityRegistrationRequest) -> Self {
        let principal_id = request.principal_id.clone();
        let principal_schema = request.principal_schema.clone();
        Self {
            id,
            identity_provider,
            principal_id,
            principal_schema,
        }
    }
}
