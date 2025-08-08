//! HTTP server implementation for reverse proxy deployment
//! 
//! This module contains the HTTP server implementation optimized for deployment
//! behind a reverse proxy (nginx, Caddy, etc.) that handles SSL, compression, and CORS.

use crate::{
    config::AppConfig,
    shared::error::{AppError, AppResult},
    infrastructure::http::{
        routes::{RouteBuilder, PaymentsRoutes},
    },
    application::{
        services::{RpcService, MetricsService},
        use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    },
    domain::{security::SecurityValidator, validation::DomainValidator, rpc::{RpcRequest, ClientInfo}},
    infrastructure::adapters::{ExternalRpcAdapter, AuthenticationAdapter, PaymentsStore, TokenIssuerAdapter, RevocationStore},
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
    },
};
use redis::{aio::ConnectionManager, Client};
use std::sync::Arc;
use tracing::{info, instrument};
use warp::{Filter, Reply};

/// HTTP server implementation optimized for reverse proxy deployment
pub struct HttpServer {
    config: AppConfig,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    metrics_use_case: Arc<GetMetricsUseCase>,
    health_use_case: Arc<HealthCheckUseCase>,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
    revocation_store: Arc<RevocationStore>,
    payments_redis: Option<Arc<ConnectionManager>>,
}

impl HttpServer {
    /// Create a new HTTP server instance optimized for reverse proxy deployment
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        // Initialize domain layer
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let _domain_validator = Arc::new(DomainValidator::new());
        
        // Initialize infrastructure layer
        let config_arc = Arc::new(config.clone());
        let _external_rpc_adapter = Arc::new(ExternalRpcAdapter::new(config_arc.clone()));
        // Revocation store setup: if cache.enabled, create Redis manager; else memory-only
        let revocation_store = if config_arc.cache.enabled {
            match Client::open(config_arc.cache.redis_url.clone()) {
                Ok(client) => match ConnectionManager::new(client).await {
                    Ok(manager) => Arc::new(RevocationStore::new(Some(Arc::new(manager)))),
                    Err(e) => { tracing::warn!("revocation redis unavailable: {} - using memory", e); Arc::new(RevocationStore::new(None)) }
                },
                Err(e) => { tracing::warn!("revocation redis client error: {} - using memory", e); Arc::new(RevocationStore::new(None)) }
            }
        } else { Arc::new(RevocationStore::new(None)) };
        let _auth_adapter = Arc::new(AuthenticationAdapter::new(config_arc.clone()).with_revocation_store(revocation_store.clone()));

        // Optional: import viewing keys at startup
        if !config_arc.payments.viewing_keys.is_empty() {
            Self::import_viewing_keys(config_arc.clone(), _external_rpc_adapter.clone()).await.ok();
        } else if config_arc.payments.require_viewing_key {
            tracing::warn!("payments.require_viewing_key=true but no viewing_keys configured");
        }
        
        // Initialize application layer
        let rpc_service = Arc::new(RpcService::new(config_arc.clone(), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        // Initialize use cases
        let rpc_use_case = Arc::new(ProcessRpcRequestUseCase::new(
            rpc_service.clone(),
            metrics_service.clone(),
        ));
        let metrics_use_case = Arc::new(GetMetricsUseCase::new(metrics_service));
        let health_use_case = Arc::new(HealthCheckUseCase);

        // Initialize cache middleware
        let cache_middleware = Arc::new(CacheMiddleware::new(&config).await?);

        // Initialize rate limiting middleware
        let rate_limit_middleware = Arc::new(RateLimitMiddleware::new(config.clone()));

        // Prepare payments Redis manager if available
        let payments_redis = if config_arc.cache.enabled {
            match Client::open(config_arc.cache.redis_url.clone()) {
                Ok(client) => match ConnectionManager::new(client).await {
                    Ok(manager) => Some(Arc::new(manager)),
                    Err(e) => { tracing::warn!("payments redis unavailable: {} - using memory", e); None }
                },
                Err(e) => { tracing::warn!("payments redis client error: {} - using memory", e); None }
            }
        } else { None };

        Ok(Self {
            config,
            rpc_use_case,
            metrics_use_case,
            health_use_case,
            cache_middleware,
            rate_limit_middleware,
            revocation_store,
            payments_redis,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Run the HTTP server optimized for reverse proxy deployment
    #[instrument(skip(self))]
    pub async fn run(self) -> AppResult<()> {
        let addr = self.config.server_address();
        info!("Starting HTTP server optimized for reverse proxy deployment on {}", addr);
        info!("SSL/TLS, compression, and CORS should be handled by the reverse proxy");
        
        let addr: std::net::SocketAddr = addr.parse()
            .map_err(|e| AppError::Config(format!("Invalid server address: {}", e)))?;

        let routes = self.create_routes();
        
        info!("Starting HTTP server (reverse proxy mode)");
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// Create the application routes optimized for reverse proxy deployment
    fn create_routes(self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let base = RouteBuilder::build_routes(
            self.config.clone(),
            self.rpc_use_case,
            self.metrics_use_case,
            self.health_use_case,
            self.cache_middleware.clone(),
            self.rate_limit_middleware.clone(),
        );

        let payments_config = crate::application::services::payments_service::PaymentsConfig::default();
        let external_rpc = std::sync::Arc::new(ExternalRpcAdapter::new(std::sync::Arc::new(self.config.clone())));
        let payments_store = std::sync::Arc::new(PaymentsStore::new(self.payments_redis.clone()));
        let token_issuer = std::sync::Arc::new(TokenIssuerAdapter::new(std::sync::Arc::new(self.config.clone())));
        let payments_service = std::sync::Arc::new(crate::application::services::payments_service::PaymentsService::new(
            std::sync::Arc::new(self.config.clone()),
            payments_config,
            external_rpc,
            payments_store,
            token_issuer,
            self.revocation_store.clone(),
        ));
        let payments_routes = PaymentsRoutes::create_routes(self.config.clone(), payments_service);

        base.or(payments_routes)
    }

    /// Import viewing keys from configuration into the wallet (non-fatal on errors)
    async fn import_viewing_keys(config: Arc<AppConfig>, rpc: Arc<ExternalRpcAdapter>) -> AppResult<()> {
        let rescan = config.payments.viewing_key_rescan.clone();
        for (idx, vkey) in config.payments.viewing_keys.iter().enumerate() {
            let client_info = ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: Some("startup".to_string()),
                auth_token: None,
                timestamp: chrono::Utc::now(),
            };
            let params = serde_json::Value::Array(vec![
                serde_json::Value::String(vkey.clone()),
                serde_json::Value::String(rescan.clone()),
            ]);
            let req = RpcRequest::new(
                "z_importviewingkey".to_string(),
                Some(params),
                Some(serde_json::json!(format!("vk_import_{}", idx))),
                client_info,
            );
            match rpc.send_request(&req).await {
                Ok(_) => tracing::info!("Imported viewing key {}", idx),
                Err(e) => tracing::warn!("Viewing key import failed ({}): {}", idx, e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
/// Create test routes for integration testing
pub async fn create_test_routes() -> Result<impl Filter<Extract = impl Reply> + Clone, Box<dyn std::error::Error + Send + Sync>> {
    // For testing, we'll create a simple configuration
    let mut config = AppConfig::default();
    config.server.port = 0; // Use random port
    config.server.bind_address = "127.0.0.1".parse().unwrap();
    config.security.development_mode = true;
    config.cache.enabled = false;
    config.rate_limit.enabled = false;
    
    let server = HttpServer::new(config).await?;
    Ok(server.create_routes())
} 