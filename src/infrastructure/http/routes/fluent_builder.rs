//! Fluent route builder module
//! 
//! This module contains a fluent API for building routes in a readable and
//! maintainable way, providing a more intuitive interface for route construction.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        routes::MiddlewareConfig,
        utils::{
            with_rpc_use_case, with_health_use_case, with_metrics_use_case,
            with_mining_pool_client, with_config, with_cache_middleware, with_rate_limit_middleware,
            with_prometheus_adapter,
        },
        handlers::{
            handle_rpc_request, handle_health_request, handle_metrics_request,
            handle_prometheus_request, handle_mining_pool_request, handle_pool_metrics_request,
        },
    },
    application::use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    middleware::{cache::CacheMiddleware, rate_limit::RateLimitMiddleware},
};
use std::sync::Arc;
use warp::Filter;

/// Fluent route builder for creating routes with a readable API
pub struct FluentRouteBuilder {
    config: AppConfig,
    cache_middleware: Option<Arc<CacheMiddleware>>,
    rate_limit_middleware: Option<Arc<RateLimitMiddleware>>,
    rpc_use_case: Option<Arc<ProcessRpcRequestUseCase>>,
    health_use_case: Option<Arc<HealthCheckUseCase>>,
    metrics_use_case: Option<Arc<GetMetricsUseCase>>,
}

impl FluentRouteBuilder {
    /// Create a new fluent route builder
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
            cache_middleware: None,
            rate_limit_middleware: None,
            rpc_use_case: None,
            health_use_case: None,
            metrics_use_case: None,
        }
    }

    /// Add config to the builder
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    /// Add cache middleware to the builder
    pub fn with_cache_middleware(mut self, cache_middleware: Arc<CacheMiddleware>) -> Self {
        self.cache_middleware = Some(cache_middleware);
        self
    }

    /// Add rate limit middleware to the builder
    pub fn with_rate_limit_middleware(mut self, rate_limit_middleware: Arc<RateLimitMiddleware>) -> Self {
        self.rate_limit_middleware = Some(rate_limit_middleware);
        self
    }

    /// Add RPC use case to the builder
    pub fn with_rpc_use_case(mut self, rpc_use_case: Arc<ProcessRpcRequestUseCase>) -> Self {
        self.rpc_use_case = Some(rpc_use_case);
        self
    }

    /// Add health use case to the builder
    pub fn with_health_use_case(mut self, health_use_case: Arc<HealthCheckUseCase>) -> Self {
        self.health_use_case = Some(health_use_case);
        self
    }

    /// Add metrics use case to the builder
    pub fn with_metrics_use_case(mut self, metrics_use_case: Arc<GetMetricsUseCase>) -> Self {
        self.metrics_use_case = Some(metrics_use_case);
        self
    }

    /// Build RPC route with fluent API
    pub fn build_rpc_route(&self) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, String> {
        let rpc_use_case = self.rpc_use_case.as_ref()
            .ok_or("RPC use case is required for RPC route")?;
        let cache_middleware = self.cache_middleware.as_ref()
            .ok_or("Cache middleware is required for RPC route")?;
        let rate_limit_middleware = self.rate_limit_middleware.as_ref()
            .ok_or("Rate limit middleware is required for RPC route")?;

        let route = warp::path::end()
            .and(warp::post())
            .and(warp::body::content_length_limit(self.config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(warp::header::optional::<String>("authorization"))
            .and(warp::header::optional::<String>("user-agent"))
            .and(with_rpc_use_case(rpc_use_case.clone()))
            .and(with_config(self.config.clone()))
            .and(with_cache_middleware(cache_middleware.clone()))
            .and(with_rate_limit_middleware(rate_limit_middleware.clone()))
            .and_then(handle_rpc_request);

        Ok(route)
    }

    /// Build health route with fluent API
    pub fn build_health_route(&self) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, String> {
        let health_use_case = self.health_use_case.as_ref()
            .ok_or("Health use case is required for health route")?;

        let route = warp::path("health")
            .and(warp::get())
            .and(with_health_use_case(health_use_case.clone()))
            .and(with_config(self.config.clone()))
            .and_then(handle_health_request);

        Ok(route)
    }

    /// Build metrics route with fluent API
    pub fn build_metrics_route(&self) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, String> {
        let metrics_use_case = self.metrics_use_case.as_ref()
            .ok_or("Metrics use case is required for metrics route")?;

        let route = warp::path("metrics")
            .and(warp::get())
            .and(with_metrics_use_case(metrics_use_case.clone()))
            .and(with_config(self.config.clone()))
            .and_then(handle_metrics_request);

        Ok(route)
    }

    /// Build Prometheus route with fluent API
    pub fn build_prometheus_route(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("prometheus")
            .and(warp::get())
            .and(with_prometheus_adapter())
            .and(with_config(self.config.clone()))
            .and_then(handle_prometheus_request)
    }

    /// Build mining pool route with fluent API
    pub fn build_mining_pool_route(&self) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, String> {
        let cache_middleware = self.cache_middleware.as_ref()
            .ok_or("Cache middleware is required for mining pool route")?;
        let rate_limit_middleware = self.rate_limit_middleware.as_ref()
            .ok_or("Rate limit middleware is required for mining pool route")?;

        let route = warp::path("pool")
            .and(warp::path("share"))
            .and(warp::post())
            .and(warp::body::content_length_limit(self.config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(with_mining_pool_client())
            .and(with_config(self.config.clone()))
            .and(with_cache_middleware(cache_middleware.clone()))
            .and(with_rate_limit_middleware(rate_limit_middleware.clone()))
            .and_then(handle_mining_pool_request);

        Ok(route)
    }

    /// Build pool metrics route with fluent API
    pub fn build_pool_metrics_route(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("pool")
            .and(warp::path("metrics"))
            .and(warp::get())
            .and(with_mining_pool_client())
            .and(with_config(self.config.clone()))
            .and_then(handle_pool_metrics_request)
    }

    /// Build all routes with fluent API
    pub fn build(&self) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, String> {
        let rpc_route = self.build_rpc_route()?;
        let health_route = self.build_health_route()?;
        let metrics_route = self.build_metrics_route()?;
        let prometheus_route = self.build_prometheus_route();
        let mining_pool_route = self.build_mining_pool_route()?;
        let pool_metrics_route = self.build_pool_metrics_route();

        let all_routes = rpc_route
            .or(health_route)
            .or(metrics_route)
            .or(prometheus_route)
            .or(mining_pool_route)
            .or(pool_metrics_route);

        Ok(all_routes)
    }

    /// Validate the builder configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check required dependencies for RPC route
        if self.rpc_use_case.is_none() {
            return Err("RPC use case is required".to_string());
        }
        if self.cache_middleware.is_none() {
            return Err("Cache middleware is required".to_string());
        }
        if self.rate_limit_middleware.is_none() {
            return Err("Rate limit middleware is required".to_string());
        }

        // Check required dependencies for health route
        if self.health_use_case.is_none() {
            return Err("Health use case is required".to_string());
        }

        // Check required dependencies for metrics route
        if self.metrics_use_case.is_none() {
            return Err("Metrics use case is required".to_string());
        }

        // Validate config
        if self.config.server.max_request_size == 0 {
            return Err("Max request size must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Convenience functions for common route building patterns
pub struct FluentRouteUtils;

impl FluentRouteUtils {
    /// Create a complete route builder with all dependencies
    pub fn create_complete_builder(
        config: AppConfig,
        rpc_use_case: Arc<ProcessRpcRequestUseCase>,
        health_use_case: Arc<HealthCheckUseCase>,
        metrics_use_case: Arc<GetMetricsUseCase>,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> FluentRouteBuilder {
        FluentRouteBuilder::new()
            .with_config(config)
            .with_rpc_use_case(rpc_use_case)
            .with_health_use_case(health_use_case)
            .with_metrics_use_case(metrics_use_case)
            .with_cache_middleware(cache_middleware)
            .with_rate_limit_middleware(rate_limit_middleware)
    }

    /// Create a minimal route builder for basic routes
    pub fn create_minimal_builder(
        config: AppConfig,
        health_use_case: Arc<HealthCheckUseCase>,
        metrics_use_case: Arc<GetMetricsUseCase>,
    ) -> FluentRouteBuilder {
        FluentRouteBuilder::new()
            .with_config(config)
            .with_health_use_case(health_use_case)
            .with_metrics_use_case(metrics_use_case)
    }

    /// Create a route builder from middleware configuration
    pub fn from_middleware_config(
        middleware_config: MiddlewareConfig,
        rpc_use_case: Option<Arc<ProcessRpcRequestUseCase>>,
        health_use_case: Option<Arc<HealthCheckUseCase>>,
        metrics_use_case: Option<Arc<GetMetricsUseCase>>,
    ) -> Result<FluentRouteBuilder, String> {
        let config = middleware_config.config.clone();
        let mut builder = FluentRouteBuilder::new()
            .with_config(config);

        if let Some(cache) = middleware_config.get_cache() {
            builder = builder.with_cache_middleware(cache.clone());
        }

        if let Some(rate_limit) = middleware_config.get_rate_limit() {
            builder = builder.with_rate_limit_middleware(rate_limit.clone());
        }

        if let Some(rpc_use_case) = rpc_use_case {
            builder = builder.with_rpc_use_case(rpc_use_case);
        }

        if let Some(health_use_case) = health_use_case {
            builder = builder.with_health_use_case(health_use_case);
        }

        if let Some(metrics_use_case) = metrics_use_case {
            builder = builder.with_metrics_use_case(metrics_use_case);
        }

        Ok(builder)
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
    async fn test_fluent_builder_build_routes() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // This should not panic and should return a valid filter
        let routes = FluentRouteBuilder::new()
            .with_config(config)
            .with_rpc_use_case(rpc_use_case)
            .with_metrics_use_case(metrics_use_case)
            .with_health_use_case(health_use_case)
            .with_cache_middleware(cache_middleware)
            .with_rate_limit_middleware(rate_limit_middleware)
            .build();
        assert!(routes.is_ok());
    }

    #[tokio::test]
    async fn test_fluent_builder_with_optional_components() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();

        // Test building without middleware
        let routes = FluentRouteBuilder::new()
            .with_config(config)
            .with_rpc_use_case(rpc_use_case)
            .with_metrics_use_case(metrics_use_case)
            .with_health_use_case(health_use_case)
            .build();
        assert!(routes.is_err());
    }

    #[tokio::test]
    async fn test_fluent_builder_with_all_components() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let metrics_use_case = create_test_metrics_use_case();
        let health_use_case = create_test_health_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Test building with all components
        let routes = FluentRouteBuilder::new()
            .with_config(config)
            .with_rpc_use_case(rpc_use_case)
            .with_metrics_use_case(metrics_use_case)
            .with_health_use_case(health_use_case)
            .with_cache_middleware(cache_middleware)
            .with_rate_limit_middleware(rate_limit_middleware)
            .build();
        assert!(routes.is_ok());
    }
}
