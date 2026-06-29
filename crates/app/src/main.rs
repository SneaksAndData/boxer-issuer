use boxer_issuer_http::services::configuration::base::initialization_configuration_manager::InitializationConfigurationManager;
use boxer_issuer_http::services::token_service::TokenService;
use log::info;
use std::sync::Arc;

use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::audit::AuditService;
use boxer_core::services::backends::kubernetes::kubernetes_repository::schema_repository::SchemaRepository;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry;
use boxer_core::services::observability::open_telemetry::metrics::init_metrics;
use boxer_core::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use boxer_core::services::observability::open_telemetry::tracing::init_tracer;
use boxer_issuer_http::services::backends::base::load_backend;
use boxer_issuer_http::services::backends::kubernetes::identity_repository::IdentityRepository;
use boxer_issuer_http::services::backends::kubernetes::principal_repository::PrincipalRepository;
use boxer_issuer_http::services::configuration::models::AppSettings;
use boxer_issuer_http::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use boxer_issuer_http::services::principal_service::PrincipalService;
use env_filter::Builder;

const ROOT_METRICS_NAMESPACE: &str = "boxer-issuer";

#[actix_web::main]
async fn main() -> Result<()> {
    let mut builder = Builder::new();

    let filter = if let Ok(ref filter) = std::env::var("RUST_LOG") {
        builder.parse(filter);
        builder.build()
    } else {
        Builder::default().parse("info").build()
    };

    let cm = AppSettings::new()?;

    let logger = ComposedLogger::new();
    let logger = {
        if cm.opentelemetry.log_settings.enabled {
            logger.with_logger(open_telemetry::logging::init_logger(cm.deploy_environment.clone())?)
        } else {
            logger
        }
    };

    logger
        .with_logger(Box::new(env_logger::Builder::from_default_env().build()))
        .with_global_level(filter)
        .init()?;

    info!("Configuration manager started");

    if cm.opentelemetry.tracing_settings.enabled {
        info!("Tracing is enabled, starting tracer...");
        init_tracer()?;
    }

    if cm.opentelemetry.metrics_settings.enabled {
        info!("Metrics is enabled, starting metrics...");
        init_metrics()?;
    }

    let current_backend = load_backend(cm.get_backend_type(), &cm).await?;
    let readiness_state = current_backend.readiness_state();

    let validator_provider: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync> = current_backend.get();

    let schemas_repository: Arc<SchemaRepository> = current_backend.get();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get();
    let identity_repository: Arc<IdentityRepository> = current_backend.get();

    let principal_service = Arc::new(PrincipalService::new(
        identity_repository.clone(),
        entities_repository.clone(),
        schemas_repository.clone(),
    ));

    let token_provider = Arc::new(TokenService::new(
        validator_provider.clone(),
        principal_service.clone(),
        cm.get_signing_key(),
        cm.get_key_id(),
        cm.get_audience(),
        cm.get_issuer(),
        cm.get_content_encryption(),
        MetricsProvider::new(ROOT_METRICS_NAMESPACE, cm.instance_name.clone()),
    ));

    // The audit_service variable is deprecated, use audit_writer instead.
    // Will be removed in the next releases.
    let audit_service: Arc<dyn AuditService> = Arc::new(LogAuditService::new());

    let audit_writer: Arc<dyn AuditWriter> = Arc::new(LogAuditService::new());

    info!(host:? = &cm.listen_address.ip(); "listening on {}:{}", &cm.listen_address.ip(), &cm.listen_address.port());

    let server = boxer_issuer_http::start_api_server(
        current_backend,
        token_provider,
        audit_service,
        audit_writer,
        readiness_state,
        principal_service,
        cm,
    )?;

    server.await.map_err(anyhow::Error::from)
}
