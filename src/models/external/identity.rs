#[derive(Debug, PartialEq)]
/// Struct that represents an external identity
pub struct ExternalIdentity {
    /// The user ID extracted from the external identity provider
    pub user_id: String,
    
    /// The name of the external identity provider
    pub identity_provider: String,
}

