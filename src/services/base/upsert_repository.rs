use crate::models::external::identity::{ExternalIdentity, Policy, PolicyAttachment};
use async_trait::async_trait;
use cedar_policy::Entities;

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
}
#[allow(dead_code)]
pub type IdentityRepository = dyn UpsertRepository<ExternalIdentity, (String, String), Error = anyhow::Error>;

pub type PolicyRepository = dyn UpsertRepository<Policy, String, Error = anyhow::Error>;

pub type PolicyAttachmentRepository = dyn UpsertRepository<PolicyAttachment, ExternalIdentity, Error = anyhow::Error>;

pub type SchemaRepository = dyn UpsertRepository<Entities, String, Error = anyhow::Error>;
