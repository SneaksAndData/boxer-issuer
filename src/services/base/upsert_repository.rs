use async_trait::async_trait;
use crate::models::external::identity::{ExternalIdentity, Policy, PolicyAttachment};

#[async_trait]
#[allow(dead_code)]
/// Represents a repository for policies
pub trait UpsertRepository<Entity, Key> {
    type Error;

    /// Retrieves a policy by id
    async fn get(&self, key: Key) -> Result<Entity, Self::Error>;

    /// Updates or inserts a policy by id
    async fn upsert(&mut self, key: Key, entity: Entity) -> Result<(), Self::Error>;

    /// Deletes policy by id
    async fn delete(&mut self, key: Key) -> Result<(), Self::Error>;
}
#[allow(dead_code)]
pub type IdentityRepository = dyn UpsertRepository<ExternalIdentity, (String, String), Error=anyhow::Error> + Send + Sync;
pub type PolicyRepository = dyn UpsertRepository<Policy, String, Error=anyhow::Error> + Send + Sync;
pub type PolicyAttachmentRepository = dyn UpsertRepository<PolicyAttachment, ExternalIdentity, Error=anyhow::Error> + Send + Sync;

