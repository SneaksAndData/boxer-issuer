use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Struct that represents an external identity
pub struct ExternalIdentity {
    /// The user ID extracted from the external identity provider
    pub user_id: String,

    /// The name of the external identity provider
    pub identity_provider: String,
}

impl ExternalIdentity {
    /// Creates a new instance of an external identity
    pub fn new(user_id: String, identity_provider: String) -> Self {
        ExternalIdentity {
            user_id: user_id.to_lowercase(),
            identity_provider: identity_provider.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PolicyAttachment {
    pub external_identity: ExternalIdentity,
    pub policies: HashSet<String>,
}

#[allow(dead_code)]
impl PolicyAttachment {
   pub fn new(external_identity: ExternalIdentity, policies: HashSet<String>) -> Self {
       PolicyAttachment {
           external_identity,
           policies,
       }
   } 
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub content: String,
}

#[allow(dead_code)]
impl Policy {
    pub fn new(content: String) -> Self {
        Policy {
            content,
        }
    }
    
    pub fn empty() -> Self {
        Policy {
            content: String::new(),
        }
    }
    
    pub fn merge(&self, other: Policy) -> Self {
        Policy {
            content: format ! ("{}\n{}", self.content, other.content),
        }
    }
}
