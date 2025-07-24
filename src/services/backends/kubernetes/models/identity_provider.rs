use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct PrincipalAssociation {
    pub schema: String,
    pub principal: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ExternalIdentityInfo {
    pub name: String,
    pub principal: Option<PrincipalAssociation>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct IdentitySetData {
    pub active: Vec<ExternalIdentityInfo>,
    pub inactive: Vec<ExternalIdentityInfo>,
}

impl IdentitySetData {
    pub fn contains(&self, user: &str) -> bool {
        self.active.iter().any(|info| info.name == user)
    }

    pub fn is_deleted(&self, user: &str) -> bool {
        self.inactive.iter().any(|info| info.name == user)
    }

    pub fn insert(&mut self, user_id: String) {
        if !self.contains(&user_id) {
            self.active.push(ExternalIdentityInfo {
                name: user_id,
                principal: Default::default(),
            });
        }
    }

    pub fn get_active(&self) -> HashSet<String> {
        self.active.iter().map(|info| info.name.clone()).collect()
    }

    pub fn remove(&mut self, user: &str) -> bool {
        if let Some(pos) = self.active.iter().position(|info| info.name == user) {
            let info = self.active.remove(pos);
            self.inactive.push(info);
            true
        } else {
            false
        }
    }
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "IdentityProvider",
    plural = "identity-providers",
    singular = "identity-provider",
    namespaced
)]
pub struct IdentityProviderSpec {
    pub identities: IdentitySetData,
}

impl Default for IdentityProvider {
    fn default() -> Self {
        IdentityProvider {
            metadata: ObjectMeta::default(),
            spec: IdentityProviderSpec {
                identities: IdentitySetData {
                    inactive: Default::default(),
                    active: Default::default(),
                },
            },
        }
    }
}
