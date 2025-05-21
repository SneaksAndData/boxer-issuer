use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
/// Struct that represents an external identity
pub struct ExternalIdentity {
    /// The user ID extracted from the external identity provider
    pub user_id: String,

    /// The name of the external identity provider
    pub identity_provider: String,
}

impl ExternalIdentity {
    /// Creates a new instance of an external identity
    pub fn new(identity_provider: String, user_id: String) -> Self {
        ExternalIdentity {
            user_id: user_id.to_lowercase(),
            identity_provider: identity_provider.to_lowercase(),
        }
    }
}

impl From<(String, String)> for ExternalIdentity {
    fn from(value: (String, String)) -> Self {
        ExternalIdentity::new(value.0, value.1)
    }
}
