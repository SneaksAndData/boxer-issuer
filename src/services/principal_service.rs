use crate::models::api::external::identity::ExternalIdentity;
use crate::models::principal::Principal;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use cedar_policy::{EntityUid, SchemaFragment};
use std::str::FromStr;
use std::sync::Arc;

pub struct PrincipalService {
    identities: Arc<IdentityRepository>,
    principals: Arc<PrincipalRepository>,
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
        schema_repository: Arc<SchemaRepository>,
    ) -> Self {
        Self {
            identities,
            principals,
            schema_repository,
        }
    }

    pub async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal, anyhow::Error> {
        let registration = self
            .identities
            .get((external_identity.identity_provider, external_identity.user_id))
            .await?;
        let uid = EntityUid::from_str(registration.principal_id.as_str())?;
        let schema_id = registration.principal_schema.clone();
        let pid = PrincipalIdentity::new(registration.principal_schema, uid);
        let entity = self.principals.get(pid).await?;
        Ok(Principal::new(entity.into(), schema_id))
    }

    pub async fn get_validator_schema(&self, external_identity: ExternalIdentity) -> Result<String, anyhow::Error> {
        let registration = self
            .identities
            .get((external_identity.identity_provider, external_identity.user_id))
            .await?;
        Ok(registration.validator_schema)
    }
}
