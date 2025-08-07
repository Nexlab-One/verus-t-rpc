//! Route builder module
//! 
//! This module contains the main route builder that orchestrates the creation
//! of all application routes.

use crate::{
    config::AppConfig,
    infrastructure::http::routes::{
        RpcRoutes, HealthRoutes, MetricsRoutes, MiningPoolRoutes,
    },
    application::use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    middleware::{cache::CacheMiddleware, rate_limit::RateLimitMiddleware},
};
use std::sync::Arc;
use warp::Filter;

/// Route builder that orchestrates the creation of all application routes
pub struct RouteBuilder;

impl RouteBuilder {
    /// Build all application routes
    pub fn build_routes(
        config: AppConfig,
        rpc_use_case: Arc<ProcessRpcRequestUseCase>,
        metrics_use_case: Arc<GetMetricsUseCase>,
        health_use_case: Arc<HealthCheckUseCase>,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // Build individual route groups
        let rpc_route = RpcRoutes::create_rpc_route(
            config.clone(),
            rpc_use_case,
            cache_middleware.clone(),
            rate_limit_middleware.clone(),
        );

        let health_route = HealthRoutes::create_health_route(
            config.clone(),
            health_use_case,
        );

        let metrics_route = MetricsRoutes::create_metrics_route(
            config.clone(),
            metrics_use_case,
        );

        let prometheus_route = MetricsRoutes::create_prometheus_route(
            config.clone(),
        );

        let mining_pool_route = MiningPoolRoutes::create_mining_pool_route(
            config.clone(),
            cache_middleware,
            rate_limit_middleware,
        );

        let pool_metrics_route = MiningPoolRoutes::create_pool_metrics_route(
            config,
        );

        // Combine all routes
        rpc_route
            .or(health_route)
            .or(metrics_route)
            .or(prometheus_route)
            .or(mining_pool_route)
            .or(pool_metrics_route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase};
    use crate::application::services::{RpcService, MetricsService};
    use crate::domain::security::SecurityPolicy;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    async fn create_test_cache_middleware() -> Arc<CacheMiddleware> {
        Arc::new(CacheMiddleware::new(&create_test_config()).await.unwrap())
    }

    fn create_test_rate_limit_middleware() -> Arc<RateLimitMiddleware> {
        Arc::new(RateLimitMiddleware::new(create_test_config()))
    }

    fn create_test_rpc_use_case() -> Arc<ProcessRpcRequestUseCase> {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(crate::domain::security::SecurityValidator::new(SecurityPolicy::default()));
        let rpc_service = Arc::new(RpcService::new(config.clone(), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(ProcessRpcRequestUseCase::new(rpc_service, metrics_service))
    }

    fn create_test_health_use_case() -> Arc<HealthCheckUseCase> {
        Arc::new(HealthCheckUseCase)
    }

    fn create_test_metrics_use_case() -> Arc<GetMetricsUseCase> {
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(GetMetricsUseCase::new(metrics_service))
    }

    #[tokio::test]
    async fn test_route_builder_build_routes() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // This should not panic and should return a valid filter
        let routes = RouteBuilder::build_routes(
            config,
            rpc_use_case,
            metrics_use_case,
            health_use_case,
            cache_middleware,
            rate_limit_middleware,
        );
        let _ = routes.clone();
    }

    #[tokio::test]
    async fn test_route_builder_creates_all_route_types() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Test that we can create individual route types
        let _rpc_route = RpcRoutes::create_rpc_route(
            config.clone(),
            rpc_use_case.clone(),
            cache_middleware.clone(),
            rate_limit_middleware.clone(),
        );

        let _health_route = HealthRoutes::create_health_route(
            config.clone(),
            health_use_case.clone(),
        );

        let _metrics_route = MetricsRoutes::create_metrics_route(
            config.clone(),
            metrics_use_case.clone(),
        );

        let _prometheus_route = MetricsRoutes::create_prometheus_route(
            config.clone(),
        );

        let _mining_pool_route = MiningPoolRoutes::create_mining_pool_route(
            config.clone(),
            cache_middleware.clone(),
            rate_limit_middleware.clone(),
        );

        let _pool_metrics_route = MiningPoolRoutes::create_pool_metrics_route(
            config.clone(),
        );

        let _ = _rpc_route.clone();
        let _ = _health_route.clone();
        let _ = _metrics_route.clone();
        let _ = _prometheus_route.clone();
        let _ = _mining_pool_route.clone();
        let _ = _pool_metrics_route.clone();
    }

    #[tokio::test]
    async fn test_route_builder_with_different_configs() {
        let mut config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Test with different configurations
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let routes = RouteBuilder::build_routes(
            config,
            rpc_use_case,
            metrics_use_case,
            health_use_case,
            cache_middleware,
            rate_limit_middleware,
        );
        let _ = routes.clone();
    }
}
