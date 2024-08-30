use crate::models::external::identity::Policy;
use crate::models::external::identity_provider::ExternalIdentityProvider;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use jwt::Claims;
use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents an internal JWT Token issued by `boxer-issuer`
pub struct InternalToken {
    pub policy: Policy,
    pub metadata: TokenMetadata,
    version: String,
}

pub struct TokenMetadata {
    pub user_id: String,
    pub identity_provider: ExternalIdentityProvider,
}

impl InternalToken {
    pub fn new(policy: Policy, user_id: String, external_identity_provider: String) -> Self {
        InternalToken {
            policy,
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
        const POLICY_KEY: &str = "boxer.sneaksanddata.com/policy";
        const USER_ID_KEY: &str = "boxer.sneaksanddata.com/user-id";
        const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";

        // The constants below to be moved in the service configuration file in the future.
        const BOXER_ISSUER: &str = "boxer.sneaksanddata.com";
        const BOXER_AUDIENCE: &str = "boxer.sneaksanddata.com";

        let compressed_policy = {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(self.policy.content.as_bytes())?;
            encoder.finish()?
        };

        let mut claims: Claims /* Type */ = Default::default();
        claims
            .private
            .insert(API_VERSION_KEY.to_string(), self.version.into());
        claims.private.insert(
            POLICY_KEY.to_string(),
            STANDARD.encode(&compressed_policy).into(),
        );
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
