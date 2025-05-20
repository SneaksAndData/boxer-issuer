use crate::models::api::external::identity::{ExternalIdentity, Policy, PolicyAttachment};
use crate::models::principal::Principal;
use async_trait::async_trait;
use cedar_policy::SchemaFragment;

#[async_trait]
#[allow(dead_code)]
/// Represents a repository for policies
pub trait UpsertRepository<Entity, Key>: Send + Sync {
    type Error;

    /// Retrieves a policy by id
    async fn get(&self, key: Key) -> Result<Entity, Self::Error>;

    /// Updates or inserts a policy by id
    async fn upsert(&self, key: Key, entity: Entity) -> Result<(), Self::Error>;

    /// Deletes policy by id
    async fn delete(&self, key: Key) -> Result<(), Self::Error>;

    /// Checks if an object exists
    async fn exists(&self, key: Key) -> Result<bool, Self::Error>;
}

pub type IdentityRepository = dyn UpsertRepository<ExternalIdentity, (String, String), Error = anyhow::Error>;

pub type PolicyRepository = dyn UpsertRepository<Policy, String, Error = anyhow::Error>;

pub type PolicyAttachmentRepository = dyn UpsertRepository<PolicyAttachment, ExternalIdentity, Error = anyhow::Error>;

pub type SchemaRepository = dyn UpsertRepository<SchemaFragment, String, Error = anyhow::Error>;

pub type PrincipalsRepository = dyn UpsertRepository<Principal, (String, String), Error = anyhow::Error>;
pub type PrincipalAssociationRepository =
    dyn UpsertRepository<(String, String), ExternalIdentity, Error = anyhow::Error>;
