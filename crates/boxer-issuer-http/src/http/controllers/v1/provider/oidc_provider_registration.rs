use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct OidcIdentityProviderRegistration {
    pub user_id_claim: String,
    pub discovery_url: String,
    pub issuers: Vec<String>,
    pub audiences: Vec<String>,
}
