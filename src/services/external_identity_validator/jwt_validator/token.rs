/// Represents an external JWT Token used to authorize the `ExternalIdentity` and issue an `InternalToken`
pub struct ExternalToken(String);

/// Allows `ExternalToken` to be converted to a String
impl Into<String> for ExternalToken {
    fn into(self) -> String {
        self.0.clone()
    }
}

/// Allows a String to be converted to an `ExternalToken`
impl From<String> for ExternalToken {
    fn from(token: String) -> Self {
        Self(token)
    }
}
