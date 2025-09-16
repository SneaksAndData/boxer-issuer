pub mod http;
mod models;
mod services;

use crate::services::configuration::base::initialization_configuration_manager::InitializationConfigurationManager;
use crate::services::token_service::TokenService;
use actix_web::middleware::{from_fn, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::controllers::v1;
use crate::http::openapi::ApiDoc;
use crate::services::backends::base::load_backend;
use crate::services::backends::kubernetes::identity_provider_repository::IdentityProviderRepository;
use crate::services::backends::kubernetes::identity_repository::IdentityRepository;
use crate::services::backends::kubernetes::principal_repository::PrincipalRepository;
use crate::services::configuration::models::AppSettings;
use crate::services::identity_validator_provider::ExternalIdentityValidatorProvider;
use crate::services::principal_service::PrincipalService;
use anyhow::Result;
use boxer_core::http::middleware::logging::custom_error_logging;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::audit::AuditService;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry;
use boxer_core::services::observability::open_telemetry::metrics::init_metrics;
use boxer_core::services::observability::open_telemetry::tracing::init_tracer;
use env_filter::Builder;
use opentelemetry_instrumentation_actix_web::RequestTracing;

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

    let validator_provider: Arc<dyn ExternalIdentityValidatorProvider + Send + Sync> = current_backend.get();

    let schemas_repository: Arc<SchemaRepository> = current_backend.get();
    let entities_repository: Arc<PrincipalRepository> = current_backend.get();
    let identity_repository: Arc<IdentityRepository> = current_backend.get();
    let identity_provider_repository: Arc<IdentityProviderRepository> = current_backend.get();

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
    ));

    let audit_service: Arc<dyn AuditService> = Arc::new(LogAuditService::new());

    info!(host:? = &cm.listen_address.ip(); "listening on {}:{}", &cm.listen_address.ip(), &cm.listen_address.port());
    HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(Logger::default())
            .wrap(from_fn(custom_error_logging))
            .app_data(Data::new(token_provider.clone()))
            .app_data(Data::new(principal_service.clone()))
            .app_data(Data::new(identity_repository.clone()))
            .app_data(Data::new(schemas_repository.clone()))
            .app_data(Data::new(entities_repository.clone()))
            .app_data(Data::new(identity_provider_repository.clone()))
            .app_data(Data::new(audit_service.clone()))
            .service(v1::urls())
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(cm.listen_address.clone())?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
