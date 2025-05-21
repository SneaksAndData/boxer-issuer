use crate::models::api::external::identity::ExternalIdentity;
use crate::models::principal::Principal;
use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalRepository, SchemaRepository,
};
use anyhow::bail;
use cedar_policy::SchemaFragment;
use std::sync::Arc;

pub struct IdentityAssociationRequest {
    pub external_identity_info: (String, String),
    pub principal_info: (String, String),
}

pub struct PrincipalService {
    identities: Arc<IdentityRepository>,
    principals: Arc<PrincipalRepository>,
    associations: Arc<PrincipalAssociationRepository>,
    schema_repository: Arc<SchemaRepository>,
}

impl PrincipalService {
    pub async fn get_schemas(&self, schema_id: String) -> Result<SchemaFragment, anyhow::Error> {
        let schema = self.schema_repository.get(schema_id).await?;
        Ok(schema)
    }
}

impl PrincipalService {
    pub fn new(
        identities: Arc<IdentityRepository>,
        principals: Arc<PrincipalRepository>,
        associations: Arc<PrincipalAssociationRepository>,
        schema_repository: Arc<SchemaRepository>,
    ) -> Self {
        Self {
            identities,
            principals,
            associations,
            schema_repository,
        }
    }

    pub async fn associate(&self, request: IdentityAssociationRequest) -> Result<(), anyhow::Error> {
        let external_identity = self.identities.get(request.external_identity_info).await?;
        let exists = self.principals.exists(request.principal_info.clone()).await?;
        if !exists {
            bail!(
                "Principal not found: {}/{}",
                request.principal_info.0,
                request.principal_info.1
            );
        }
        self.associations
            .upsert(external_identity.clone(), request.principal_info)
            .await
    }

    pub async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal, anyhow::Error> {
        let (principal_type, principal_id) = self.associations.get(external_identity.clone()).await?;

        let principal = self.principals.get((principal_type, principal_id)).await?;
        Ok(principal)
    }
}
