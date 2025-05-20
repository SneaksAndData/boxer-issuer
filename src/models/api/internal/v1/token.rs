use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cedar_policy::{Entity, SchemaFragment};
use jwt::Claims;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents an internal JWT Token issued by `boxer-issuer`
pub struct InternalToken {
    pub principal: Entity,
    pub schema: SchemaFragment,
    pub metadata: TokenMetadata,
    version: String,
}

pub struct TokenMetadata {
    pub user_id: String,
    pub identity_provider: ExternalIdentityProvider,
}

impl InternalToken {
    pub fn new(principal: Entity, schema: SchemaFragment, user_id: String, external_identity_provider: String) -> Self {
        InternalToken {
            principal,
            schema,
            metadata: TokenMetadata {
                user_id,
                identity_provider: ExternalIdentityProvider::from(external_identity_provider),
            },
            version: "v1".to_string(),
        }
    }
}

impl TryInto<Claims> for InternalToken {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Claims, Self::Error> {
        // This claim should be always present in the boxer token
        const API_VERSION_KEY: &str = "boxer.sneaksanddata.com/api-version";

        // Constants related to a particular API version
        const PRINCIPAL_KEY: &str = "boxer.sneaksanddata.com/principal";
        const SCHEMA_KEY: &str = "boxer.sneaksanddata.com/schema";
        const USER_ID_KEY: &str = "boxer.sneaksanddata.com/user-id";
        const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";

        // The constants below to be moved in the service configuration file in the future.
        const BOXER_ISSUER: &str = "boxer.sneaksanddata.com";
        const BOXER_AUDIENCE: &str = "boxer.sneaksanddata.com";

        let principal_json = self.principal.to_json_string()?;
        let schema_json = self.schema.to_json_string()?;

        let mut claims: Claims /* Type */ = Default::default();
        claims.private.insert(API_VERSION_KEY.to_string(), self.version.into());
        claims
            .private
            .insert(PRINCIPAL_KEY.to_string(), STANDARD.encode(&principal_json).into());
        claims
            .private
            .insert(SCHEMA_KEY.to_string(), STANDARD.encode(&schema_json).into());
        claims
            .private
            .insert(USER_ID_KEY.to_string(), self.metadata.user_id.into());
        claims.private.insert(
            IDENTITY_PROVIDER_KEY.to_string(),
            self.metadata.identity_provider.name().into(),
        );

        claims.registered.issuer = Some(BOXER_ISSUER.to_string());
        claims.registered.audience = Some(BOXER_AUDIENCE.to_string());
        let one_hour = SystemTime::now() + Duration::from_secs(3600);
        claims.registered.expiration = Some(one_hour.duration_since(UNIX_EPOCH)?.as_secs());
        Ok(claims)
    }
}
