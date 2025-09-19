use crate::http::controllers::v1::identity::external_identity_registration::ExternalIdentityRegistration;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::principal_identity::PrincipalIdentity;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use crate::services::external_identity_validator::jwt_validator::token::ExternalToken;
use crate::services::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use crate::services::external_identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::token_provider::principal::Principal;
use crate::services::token_provider::TokenProvider;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::TokenBuilder;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use cedar_policy::EntityUid;
use josekit::jwe::{Dir, JweHeader};
use josekit::jwt;
use josekit::jwt::JwtPayload;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

pub struct EncryptedTokenProvider {
    validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
    schemas: Arc<SchemaRepository>,
    identities: Arc<IdentityRepository>,
    principals: Arc<PrincipalRepository>,
    encrypt_secret: Arc<Vec<u8>>,
    audience: String,
    key_id: String,
    issuer: String,
    content_encryption: String,
}

#[async_trait]
impl TokenProvider for EncryptedTokenProvider {
    async fn issue_token(
        &self,
        provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error> {
        let validator = self.validators.get(provider.clone()).await?;
        let identity = validator.validate(external_token).await?;

        let registration = self
            .identities
            .get((identity.identity_provider.clone(), identity.user_id.clone()))
            .await?;

        let principal = self.get_principal(&registration).await?;
        let schema_name = principal.get_schema_id().clone();
        let schemas = self.schemas.get(schema_name.clone()).await?;

        let payload: JwtPayload = TokenBuilder::new()
            .principal(principal.get_entity().clone())
            .schema(schemas)
            .user_id(identity.user_id)
            .identity_provider(identity.identity_provider)
            .schema_name(schema_name)
            .validity_period(Duration::from_secs(3600))
            .validator_schema_id(registration.validator_schema)
            .build()?
            .try_into()?;

        let mut header = JweHeader::new();
        header.set_token_type("JWT");
        header.set_audience(vec![self.audience.as_str()]);
        header.set_issuer(self.issuer.clone());
        header.set_content_encryption(&self.content_encryption);
        header.set_key_id(&self.key_id);

        let encrypter = Dir.encrypter_from_bytes(&*self.encrypt_secret)?;
        jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(|e| anyhow::anyhow!(e))
    }
}

impl EncryptedTokenProvider {
    pub fn new(
        validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
        encrypt_secret: Arc<Vec<u8>>,
        key_id: String,
        audience: String,
        issuer: String,
        content_encryption: String,
        schemas: Arc<SchemaRepository>,
        identities: Arc<IdentityRepository>,
        principals: Arc<PrincipalRepository>,
    ) -> Self {
        EncryptedTokenProvider {
            validators,
            encrypt_secret,
            key_id,
            audience,
            issuer,
            content_encryption,
            schemas,
            identities,
            principals,
        }
    }

    async fn get_principal(&self, registration: &ExternalIdentityRegistration) -> Result<Principal, anyhow::Error> {
        let uid = EntityUid::from_str(registration.principal_id.as_str())?;
        let schema_id = registration.principal_schema.clone();
        let pid = PrincipalIdentity::new(registration.principal_schema.clone(), uid);
        let entity = self.principals.get(pid).await?;
        Ok(Principal::new(entity.into(), schema_id))
    }
}
