use crate::models::external::identity::ExternalIdentity;
use anyhow::bail;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
/// Represents a repository for policies
pub trait PolicyRepository {
    /// Retrieves a policy for the given external identity
    async fn get_policy(
        &self,
        external_identity: &ExternalIdentity,
    ) -> Result<String, anyhow::Error>;
}

/// Dummy implementation of a policy repository
pub struct InMemoryPolicyRepository {
    policies: Arc<HashMap<ExternalIdentity, String>>,
}

#[async_trait]
impl PolicyRepository for InMemoryPolicyRepository {
    async fn get_policy(
        &self,
        external_identity: &ExternalIdentity,
    ) -> Result<String, anyhow::Error> {
        match self.policies.get(external_identity) {
            Some(policy) => Ok(policy.clone()),
            None => bail!("Policy not found"),
        }
    }
}

impl InMemoryPolicyRepository {
    pub fn new() -> Self {
        let policy = "".to_string();
        let dummy_identity =
            ExternalIdentity::new("user@example.com".to_string(), "provider".to_string());

        let mut map = HashMap::new();
        map.insert(dummy_identity, policy.to_string());
        InMemoryPolicyRepository {
            policies: Arc::new(map),
        }
    }
}
