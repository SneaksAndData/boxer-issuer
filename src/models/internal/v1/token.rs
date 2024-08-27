use crate::models::external::identity_provider::ExternalIdentityProvider;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;

/// Represents an internal JWT Token issued by `boxer-issuer`
pub struct InternalToken {
    pub policy: String,
    pub metadata: TokenMetadata,
    version: String,
}

pub struct TokenMetadata {
    pub user_id: String,
    pub identity_provider: ExternalIdentityProvider,
}

impl InternalToken {
    pub fn new(policy: String, user_id: String, external_identity_provider: String) -> Self {
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

impl TryInto<HashMap<String, String>> for InternalToken {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<HashMap<String, String>, Self::Error> {
        const API_VERSION_KEY: &str = "boxer.sneaksanddata.com/api-version";
        const POLICY_KEY: &str = "boxer.sneaksanddata.com/policy";
        const USER_ID_KEY: &str = "boxer.sneaksanddata.com/user-id";
        const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";

        let compressed_policy = {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(self.policy.as_bytes())?;
            encoder.finish()?
        };

        let mut map = HashMap::new();
        map.insert(API_VERSION_KEY.to_string(), self.version);
        map.insert(POLICY_KEY.to_string(), STANDARD.encode(&compressed_policy));
        map.insert(USER_ID_KEY.to_string(), self.metadata.user_id);
        map.insert(
            IDENTITY_PROVIDER_KEY.to_string(),
            self.metadata.identity_provider.name(),
        );

        Ok(map)
    }
}
