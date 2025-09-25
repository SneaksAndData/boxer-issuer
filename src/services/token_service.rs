use crate::models::api::external::identity_provider::ExternalIdentityProvider;
use crate::models::api::external::token::ExternalToken;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::principal_service::PrincipalService;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::TokenBuilder;
use boxer_core::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use boxer_core::services::observability::open_telemetry::metrics::token_forbidden::{
    TokenForbidden, TokenForbiddenMetric,
};
use boxer_core::services::observability::open_telemetry::metrics::token_issued::{TokenIssued, TokenIssuedMetric};
use boxer_core::services::observability::open_telemetry::metrics::token_lifetime::{
    TokenLifetime, TokenLifetimeMetric,
};
use boxer_core::services::service_provider::ServiceProvider;
use josekit::jwe::{Dir, JweHeader};
use josekit::jwt;
use josekit::jwt::JwtPayload;
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait TokenProvider {
    async fn issue_token(
        &self,
        external_identity_provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error>;
}

pub struct TokenService {
    validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
    principal_service: Arc<PrincipalService>,
    encrypt_secret: Arc<Vec<u8>>,
    audience: String,
    key_id: String,
    issuer: String,
    content_encryption: String,
    token_duration: Duration,
    metrics_provider: MetricsProvider,
}

#[async_trait]
impl TokenProvider for TokenService {
    async fn issue_token(
        &self,
        provider: ExternalIdentityProvider,
        external_token: ExternalToken,
    ) -> Result<String, anyhow::Error> {
        let validator = self.validators.get(provider.clone()).await?;
        let identity = validator.validate(external_token).await.map_err(|e| {
            let metric: TokenForbidden = self.metrics_provider.get();
            metric.increment(provider.name().clone());
            e
        })?;
        let principal = self.principal_service.get_principal(identity.clone()).await?;
        let schema_name = principal.get_schema_id().clone();
        let schemas = self.principal_service.get_schemas(schema_name.clone()).await?;
        let validator_schema_id = self.principal_service.get_validator_schema(identity.clone()).await?;
        let payload: JwtPayload = TokenBuilder::new()
            .principal(principal.get_entity().clone())
            .schema(schemas)
            .user_id(identity.user_id.clone())
            .identity_provider(identity.identity_provider.clone())
            .schema_name(schema_name)
            .validity_period(self.token_duration.clone())
            .validator_schema_id(validator_schema_id)
            .build()?
            .try_into()?;

        let mut header = JweHeader::new();
        header.set_token_type("JWT");
        header.set_audience(vec![self.audience.as_str()]);
        header.set_issuer(self.issuer.clone());
        header.set_content_encryption(&self.content_encryption);
        header.set_key_id(&self.key_id);

        let encrypter = Dir.encrypter_from_bytes(&*self.encrypt_secret)?;
        let token = jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(|e| anyhow::anyhow!(e))?;

        let ti: TokenIssued = self.metrics_provider.get();
        ti.increment(identity.identity_provider.clone(), identity.user_id.clone());

        let tl: TokenLifetime = self.metrics_provider.get();
        tl.increment(identity.identity_provider, identity.user_id, self.token_duration);

        Ok(token)
    }
}

impl TokenService {
    pub fn new(
        validators: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync>,
        principal_service: Arc<PrincipalService>,
        encrypt_secret: Arc<Vec<u8>>,
        key_id: String,
        audience: String,
        issuer: String,
        content_encryption: String,
        metrics_provider: MetricsProvider,
    ) -> Self {
        TokenService {
            validators,
            principal_service,
            encrypt_secret,
            key_id,
            audience,
            issuer,
            content_encryption,
            token_duration: Duration::from_secs(3600),
            metrics_provider,
        }
    }
}
