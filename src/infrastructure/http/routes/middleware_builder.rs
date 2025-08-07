//! Middleware builder utilities module
//! 
//! This module contains simplified middleware composition utilities for building
//! common middleware patterns in a readable and maintainable way.

use crate::{
    config::AppConfig,
    middleware::{
        cache::CacheMiddleware,
        rate_limit::RateLimitMiddleware,
    },
};
use std::sync::Arc;

/// Middleware configuration for routes
#[derive(Clone)]
pub struct MiddlewareConfig {
    /// Cache middleware
    pub cache: Option<Arc<CacheMiddleware>>,
    /// Rate limit middleware
    pub rate_limit: Option<Arc<RateLimitMiddleware>>,
    /// App configuration
    pub config: AppConfig,
}

impl MiddlewareConfig {
    /// Create a new middleware configuration
    pub fn new(config: AppConfig) -> Self {
        Self {
            cache: None,
            rate_limit: None,
            config,
        }
    }

    /// Add cache middleware
    pub fn with_cache(mut self, cache: Arc<CacheMiddleware>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Add rate limit middleware
    pub fn with_rate_limit(mut self, rate_limit: Arc<RateLimitMiddleware>) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Check if cache middleware is configured
    pub fn has_cache(&self) -> bool {
        self.cache.is_some()
    }

    /// Check if rate limit middleware is configured
    pub fn has_rate_limit(&self) -> bool {
        self.rate_limit.is_some()
    }

    /// Get cache middleware reference
    pub fn get_cache(&self) -> Option<&Arc<CacheMiddleware>> {
        self.cache.as_ref()
    }

    /// Get rate limit middleware reference
    pub fn get_rate_limit(&self) -> Option<&Arc<RateLimitMiddleware>> {
        self.rate_limit.as_ref()
    }
}

/// Utility functions for common middleware patterns
pub struct MiddlewareUtils;

impl MiddlewareUtils {
    /// Create middleware configuration from app config and middleware
    pub fn create_middleware_config(
        config: AppConfig,
        cache_middleware: Option<Arc<CacheMiddleware>>,
        rate_limit_middleware: Option<Arc<RateLimitMiddleware>>,
    ) -> MiddlewareConfig {
        let mut middleware_config = MiddlewareConfig::new(config);
        
        if let Some(cache) = cache_middleware {
            middleware_config = middleware_config.with_cache(cache);
        }
        
        if let Some(rate_limit) = rate_limit_middleware {
            middleware_config = middleware_config.with_rate_limit(rate_limit);
        }
        
        middleware_config
    }

    /// Validate middleware configuration
    pub fn validate_middleware_config(config: &MiddlewareConfig) -> Result<(), String> {
        // Basic validation - ensure config is present
        if config.config.server.max_request_size == 0 {
            return Err("Max request size must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Get recommended middleware configuration for RPC routes
    pub fn get_rpc_middleware_config(
        config: AppConfig,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> MiddlewareConfig {
        MiddlewareConfig::new(config)
            .with_cache(cache_middleware)
            .with_rate_limit(rate_limit_middleware)
    }

    /// Get recommended middleware configuration for health routes
    pub fn get_health_middleware_config(
        config: AppConfig,
    ) -> MiddlewareConfig {
        MiddlewareConfig::new(config)
    }

    /// Get recommended middleware configuration for metrics routes
    pub fn get_metrics_middleware_config(
        config: AppConfig,
    ) -> MiddlewareConfig {
        MiddlewareConfig::new(config)
    }

    /// Get recommended middleware configuration for mining pool routes
    pub fn get_mining_pool_middleware_config(
        config: AppConfig,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> MiddlewareConfig {
        MiddlewareConfig::new(config)
            .with_cache(cache_middleware)
            .with_rate_limit(rate_limit_middleware)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    async fn create_test_cache_middleware() -> Arc<CacheMiddleware> {
        Arc::new(CacheMiddleware::new(&create_test_config()).await.unwrap())
    }

    fn create_test_rate_limit_middleware() -> Arc<RateLimitMiddleware> {
        Arc::new(RateLimitMiddleware::new(create_test_config()))
    }

    #[tokio::test]
    async fn test_middleware_config_creation() {
        let config = create_test_config();
        let middleware_config = MiddlewareConfig::new(config);

        assert!(middleware_config.cache.is_none());
        assert!(middleware_config.rate_limit.is_none());
    }

    #[tokio::test]
    async fn test_middleware_config_with_cache() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        
        let middleware_config = MiddlewareConfig::new(config)
            .with_cache(cache_middleware.clone());

        assert!(middleware_config.cache.is_some());
        assert!(middleware_config.rate_limit.is_none());
        assert!(Arc::ptr_eq(middleware_config.cache.as_ref().unwrap(), &cache_middleware));
    }

    #[tokio::test]
    async fn test_middleware_config_with_rate_limit() {
        let config = create_test_config();
        let rate_limit_middleware = create_test_rate_limit_middleware();
        
        let middleware_config = MiddlewareConfig::new(config)
            .with_rate_limit(rate_limit_middleware.clone());

        assert!(middleware_config.cache.is_none());
        assert!(middleware_config.rate_limit.is_some());
        assert!(Arc::ptr_eq(middleware_config.rate_limit.as_ref().unwrap(), &rate_limit_middleware));
    }

    #[tokio::test]
    async fn test_middleware_config_with_all_middleware() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();
        
        let middleware_config = MiddlewareConfig::new(config)
            .with_cache(cache_middleware.clone())
            .with_rate_limit(rate_limit_middleware.clone());

        assert!(middleware_config.cache.is_some());
        assert!(middleware_config.rate_limit.is_some());
        
        assert!(Arc::ptr_eq(middleware_config.cache.as_ref().unwrap(), &cache_middleware));
        assert!(Arc::ptr_eq(middleware_config.rate_limit.as_ref().unwrap(), &rate_limit_middleware));
    }

    #[tokio::test]
    async fn test_middleware_utils_create_middleware_config() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let middleware_config = MiddlewareUtils::create_middleware_config(
            config,
            Some(cache_middleware.clone()),
            Some(rate_limit_middleware.clone())
        );

        assert!(middleware_config.cache.is_some());
        assert!(middleware_config.rate_limit.is_some());
        assert!(Arc::ptr_eq(middleware_config.cache.as_ref().unwrap(), &cache_middleware));
        assert!(Arc::ptr_eq(middleware_config.rate_limit.as_ref().unwrap(), &rate_limit_middleware));
    }

    #[tokio::test]
    async fn test_middleware_utils_validate_middleware_config() {
        let config = create_test_config();
        let middleware_config = MiddlewareConfig::new(config);

        // Should pass validation
        assert!(MiddlewareUtils::validate_middleware_config(&middleware_config).is_ok());
    }
}
