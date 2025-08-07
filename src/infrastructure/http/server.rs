//! HTTP server implementation for reverse proxy deployment
//! 
//! This module contains the HTTP server implementation optimized for deployment
//! behind a reverse proxy (nginx, Caddy, etc.) that handles SSL, compression, and CORS.

use crate::{
    config::AppConfig,
    shared::error::{AppError, AppResult},
    infrastructure::http::{
        routes::RouteBuilder,
    },
    application::{
        services::{RpcService, MetricsService},
        use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    },
    domain::{security::SecurityValidator, validation::DomainValidator},
    infrastructure::adapters::{ExternalRpcAdapter, AuthenticationAdapter},
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
    },
};
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
        let _auth_adapter = Arc::new(AuthenticationAdapter::new(config_arc.clone()));
        
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

        Ok(Self {
            config,
            rpc_use_case,
            metrics_use_case,
            health_use_case,
            cache_middleware,
            rate_limit_middleware,
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
        RouteBuilder::build_routes(
            self.config,
            self.rpc_use_case,
            self.metrics_use_case,
            self.health_use_case,
            self.cache_middleware,
            self.rate_limit_middleware,
        )
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