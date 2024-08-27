#[derive(Debug, Hash, Eq, PartialEq, Clone)]
/// Struct that represents an external identity provider
pub struct ExternalIdentityProvider {
    name: String,
}

/// Converts a string into an external identity provider instance.
impl From<String> for ExternalIdentityProvider {
    fn from(name: String) -> Self {
        ExternalIdentityProvider { name }
    }
}

impl ExternalIdentityProvider {
    /// Copies the name of the external identity provider.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
