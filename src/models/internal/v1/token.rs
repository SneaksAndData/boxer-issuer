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
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(self.policy.as_bytes())?;
        let compressed = e.finish()?;

        let mut map = HashMap::new();
        map.insert(
            "boxer.sneaksanddata.com/api-version".to_string(),
            self.version,
        );
        map.insert(
            "boxer.sneaksanddata.com/policy".to_string(),
            STANDARD.encode(&compressed),
        );
        map.insert(
            "boxer.sneaksanddata.com/user-id".to_string(),
            self.metadata.user_id,
        );
        map.insert(
            "boxer.sneaksanddata.com/identity-provider".to_string(),
            self.metadata.identity_provider.name(),
        );
        Ok(map)
    }
}
