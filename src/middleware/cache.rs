//! Cache middleware for HTTP responses
//! 
//! This module provides HTTP response caching middleware to improve
//! performance and reduce load on the Verus daemon.

use crate::config::AppConfig;
use crate::infrastructure::adapters::{CacheAdapter, CacheEntry};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache middleware for HTTP responses
pub struct CacheMiddleware {
    cache_adapter: Arc<CacheAdapter>,
}

impl CacheMiddleware {
    /// Create a new cache middleware
    pub async fn new(config: &AppConfig) -> crate::Result<Self> {
        let cache_config = crate::infrastructure::adapters::CacheConfig {
            redis_url: config.cache.redis_url.clone(),
            default_ttl: config.cache.default_ttl,
            enabled: config.cache.enabled,
            max_size: config.cache.max_size,
        };
        
        let cache_adapter = Arc::new(CacheAdapter::new(cache_config).await?);
        Ok(Self { cache_adapter })
    }

    /// Check if response should be cached
    pub fn should_cache_response(&self, method: &str, status_code: u16) -> bool {
        // Only cache successful responses
        if status_code != 200 {
            return false;
        }

        // Only cache read-only methods
        self.cache_adapter.should_cache_method(method)
    }

    /// Generate cache key for method and parameters
    pub fn generate_cache_key(&self, method: &str, params: &serde_json::Value) -> String {
        self.cache_adapter.generate_cache_key(method, params)
    }

    /// Get cached response
    pub async fn get_cached_response(&self, key: &str) -> crate::Result<Option<CacheEntry>> {
        self.cache_adapter.get(key).await
    }

    /// Cache response
    pub async fn cache_response(&self, entry: CacheEntry) -> crate::Result<()> {
        self.cache_adapter.set(entry).await
    }

    /// Create cache entry
    pub fn create_cache_entry(
        &self,
        key: String,
        data: Vec<u8>,
        content_type: String,
        ttl: u64,
    ) -> CacheEntry {
        CacheEntry {
            key,
            data,
            content_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl,
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> crate::infrastructure::adapters::CacheStats {
        self.cache_adapter.get_stats().await
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> crate::Result<()> {
        self.cache_adapter.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_middleware_creation() {
        let mut config = AppConfig::default();
        config.cache.enabled = false; // Disable cache to avoid Redis connection
        let middleware = CacheMiddleware::new(&config).await.unwrap();
        
        assert!(middleware.cache_adapter.should_cache_method("getinfo"));
        assert!(!middleware.cache_adapter.should_cache_method("sendrawtransaction"));
    }

    #[tokio::test]
    async fn test_should_cache_response() {
        let mut config = AppConfig::default();
        config.cache.enabled = false; // Disable cache to avoid Redis connection
        let middleware = CacheMiddleware::new(&config).await.unwrap();
        
        // Should cache successful read-only requests
        assert!(middleware.should_cache_response("getinfo", 200));
        
        // Should not cache non-200 responses
        assert!(!middleware.should_cache_response("getinfo", 404));
        
        // Should not cache write operations
        assert!(!middleware.should_cache_response("sendrawtransaction", 200));
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let mut config = AppConfig::default();
        config.cache.enabled = false; // Disable cache to avoid Redis connection
        let middleware = CacheMiddleware::new(&config).await.unwrap();
        
        let params = serde_json::json!({"param1": "value1", "param2": "value2"});
        let key = middleware.generate_cache_key("getinfo", &params);
        
        // Check that the key follows the expected format (hash-based)
        assert!(key.starts_with("verus_rpc:"));
        assert!(key.len() > 10); // Should be a reasonable hash length
    }
} 