use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PolicyAttachment {
    pub policies: HashSet<String>,
}

#[allow(dead_code)]
impl PolicyAttachment {
    pub fn new(policies: HashSet<String>) -> Self {
        PolicyAttachment { policies }
    }

    pub fn single(policy: String) -> Self {
        let mut set = HashSet::new();
        set.insert(policy);
        PolicyAttachment { policies: set }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub content: String,
}

#[allow(dead_code)]
impl Policy {
    pub fn new(content: String) -> Self {
        Policy { content }
    }

    pub fn empty() -> Self {
        Policy {
            content: String::new(),
        }
    }

    pub fn merge(&self, other: Policy) -> Self {
        Policy {
            content: format!("{}\n{}", self.content, other.content),
        }
    }
}
