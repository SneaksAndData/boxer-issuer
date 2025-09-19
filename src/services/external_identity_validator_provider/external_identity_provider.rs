#[derive(Debug, Hash, Eq, PartialEq, Clone)]
/// Struct that represents an external identity provider
pub struct ExternalIdentityProvider(String);

/// Converts a string into an external identity provider instance.
impl From<String> for ExternalIdentityProvider {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl ExternalIdentityProvider {
    pub fn name(&self) -> String {
        self.0.clone()
    }
}
