use crate::services::base::upsert_repository::{IdentityRepository, PrincipalAssociationRepository, PrincipalsRepository, SchemaRepository};
use anyhow::bail;
use std::sync::Arc;
use cedar_policy::{Entities, SchemaFragment};
use utoipa::schema;
use crate::models::external::identity::ExternalIdentity;

pub struct IdentityAssociationRequest {
    pub external_identity_info: (String, String),
    pub principal_info: (String, String),
}

pub struct PrincipalService {
    identities: Arc<IdentityRepository>,
    principals: Arc<PrincipalsRepository>,
    associations: Arc<PrincipalAssociationRepository>,
    schema_repository: Arc<SchemaRepository>,
}

impl PrincipalService {
    pub(crate) async fn get_schemas(&self, schema_id: String) -> Result<SchemaFragment, anyhow::Error> {
        let schema = self.schema_repository.get(schema_id).await?;
        Ok(schema)
    }
}

impl PrincipalService {
    pub fn new(
        identities: Arc<IdentityRepository>,
        principals: Arc<PrincipalsRepository>,
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
    
    pub async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<(Entities, String), anyhow::Error> {
        let (principal, schema_id) = self
            .principals
            .get((external_identity.user_id.clone(), external_identity.identity_provider.clone()))
            .await?;
        Ok( (principal, schema_id) )
    }
}
