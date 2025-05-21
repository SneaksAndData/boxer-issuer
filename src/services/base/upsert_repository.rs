use crate::models::api::external::identity::ExternalIdentity;
use crate::models::principal::Principal;
use async_trait::async_trait;
use cedar_policy::SchemaFragment;

#[async_trait]
/// Represents a repository for policies
pub trait UpsertRepository<Key, Entity>: Send + Sync {
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

pub type IdentityRepository = dyn UpsertRepository<(String, String), ExternalIdentity, Error = anyhow::Error>;

pub type SchemaRepository = dyn UpsertRepository<String, SchemaFragment, Error = anyhow::Error>;

pub type PrincipalRepository = dyn UpsertRepository<(String, String), Principal, Error = anyhow::Error>;

pub type PrincipalAssociationRepository =
    dyn UpsertRepository<ExternalIdentity, (String, String), Error = anyhow::Error>;
