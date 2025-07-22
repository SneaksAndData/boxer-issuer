use crate::models::api::external::identity::ExternalIdentity;
use crate::models::principal::Principal;
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::EntityUid;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type IdentityRepository = dyn UpsertRepositoryWithDelete<
    (String, String),
    ExternalIdentity,
    Error = anyhow::Error,
    ReadError = anyhow::Error,
    DeleteError = anyhow::Error,
>;

pub type PrincipalRepository = dyn UpsertRepositoryWithDelete<
    PrincipalIdentity,
    Principal,
    Error = anyhow::Error,
    ReadError = anyhow::Error,
    DeleteError = anyhow::Error,
>;

pub type PrincipalAssociationRepository = dyn UpsertRepositoryWithDelete<
    ExternalIdentity,
    PrincipalIdentity,
    Error = anyhow::Error,
    ReadError = anyhow::Error,
    DeleteError = anyhow::Error,
>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrincipalIdentity {
    schema_id: String,
    principal_id: String,
}

impl PrincipalIdentity {
    pub fn new(schema_id: String, principal_id: String) -> Self {
        Self {
            schema_id,
            principal_id,
        }
    }

    pub fn schema_id(&self) -> &String {
        &self.schema_id
    }
    pub fn principal_id(&self) -> &String {
        &self.principal_id
    }
}

impl From<(String, String)> for PrincipalIdentity {
    fn from(tuple: (String, String)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<(String, EntityUid)> for PrincipalIdentity {
    fn from(tuple: (String, EntityUid)) -> Self {
        Self::new(tuple.0, tuple.1.to_string())
    }
}

impl Display for PrincipalIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.schema_id, self.principal_id)
    }
}
