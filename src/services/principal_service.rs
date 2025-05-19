use crate::services::base::upsert_repository::{
    IdentityRepository, PrincipalAssociationRepository, PrincipalsRepository,
};
use anyhow::bail;
use std::sync::Arc;

pub struct IdentityAssociationRequest {
    pub external_identity_info: (String, String),
    pub principal_info: (String, String),
}

pub struct PrincipalService {
    identities: Arc<IdentityRepository>,
    principals: Arc<PrincipalsRepository>,
    principal: Arc<PrincipalAssociationRepository>,
}

impl PrincipalService {
    pub fn new(
        identities: Arc<IdentityRepository>,
        principals: Arc<PrincipalsRepository>,
        principal: Arc<PrincipalAssociationRepository>,
    ) -> Self {
        Self {
            identities,
            principals,
            principal,
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
        self.principal
            .upsert(external_identity.clone(), request.principal_info)
            .await
    }
}
