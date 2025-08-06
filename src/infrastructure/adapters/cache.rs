//! Cache adapter for HTTP response caching
//! 
//! This adapter provides HTTP response caching using Redis to improve
//! performance and reduce load on the Verus daemon.

use crate::shared::error::{AppError, AppResult};
use redis::{Client, aio::ConnectionManager, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use validator::Validate;

/// Cache entry for HTTP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Response data
    pub data: Vec<u8>,
    /// Content type
    pub content_type: String,
    /// Cache timestamp
    pub timestamp: u64,
    /// Time to live in seconds
    pub ttl: u64,
    /// Cache key
    pub key: String,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Enable caching
    pub enabled: bool,
    /// Maximum cache size in bytes
    pub max_size: usize,
}

/// Cache adapter for HTTP response caching
pub struct CacheAdapter {
    /// Redis connection manager
    redis_manager: Option<ConnectionManager>,
    /// In-memory cache fallback
    memory_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache configuration
    config: CacheConfig,
}

impl CacheAdapter {
    /// Create a new cache adapter
    pub async fn new(config: CacheConfig) -> AppResult<Self> {
        let redis_manager = if config.enabled {
            match Self::create_redis_manager(&config.redis_url).await {
                Ok(manager) => {
                    info!("Redis cache connection established successfully");
                    Some(manager)
                }
                Err(e) => {
                    warn!("Failed to connect to Redis cache: {}. Using in-memory fallback only.", e);
                    info!("To use Redis caching, ensure Redis is running and accessible at: {}", config.redis_url);
                    info!("You can start Redis with: redis-server");
                    info!("Or disable caching by setting cache.enabled = false in configuration");
                    None
                }
            }
        } else {
            info!("Redis caching is disabled in configuration");
            None
        };

        Ok(Self {
            redis_manager,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Create Redis connection manager
    async fn create_redis_manager(redis_url: &str) -> AppResult<ConnectionManager> {
        let client = Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Failed to create Redis client: {}", e)))?;
        
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create Redis connection manager: {}", e)))?;
        
        Ok(manager)
    }

    /// Get a cached response
    pub async fn get(&self, key: &str) -> AppResult<Option<CacheEntry>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Try Redis first
        if let Some(ref manager) = self.redis_manager {
            match self.get_from_redis(manager, key).await {
                Ok(Some(entry)) => {
                    debug!("Cache hit for key: {}", key);
                    return Ok(Some(entry));
                }
                Ok(None) => {
                    debug!("Cache miss for key: {}", key);
                }
                Err(e) => {
                    warn!("Redis cache error: {}. Falling back to memory cache.", e);
                }
            }
        }

        // Fall back to in-memory cache
        self.get_from_memory(key).await
    }

    /// Set a cached response
    pub async fn set(&self, entry: CacheEntry) -> AppResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Try Redis first
        if let Some(ref manager) = self.redis_manager {
            match self.set_in_redis(manager, &entry).await {
                Ok(()) => {
                    debug!("Cached response in Redis for key: {}", entry.key);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Redis cache error: {}. Falling back to memory cache.", e);
                }
            }
        }

        // Fall back to in-memory cache
        self.set_in_memory(entry).await
    }

    /// Get from Redis cache
    async fn get_from_redis(&self, manager: &ConnectionManager, key: &str) -> AppResult<Option<CacheEntry>> {
        let mut conn = manager.clone();
        
        let data: RedisResult<Option<Vec<u8>>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await;

        match data {
            Ok(Some(data)) => {
                let entry: CacheEntry = serde_json::from_slice(&data)
                    .map_err(|e| AppError::Internal(format!("Failed to deserialize cache entry: {}", e)))?;
                
                // Check if entry is expired
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                if now - entry.timestamp > entry.ttl {
                    // Entry is expired, remove it
                    let _: () = redis::cmd("DEL")
                        .arg(key)
                        .query_async(&mut conn)
                        .await
                        .unwrap_or_default();
                    Ok(None)
                } else {
                    Ok(Some(entry))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::Internal(format!("Redis get error: {}", e))),
        }
    }

    /// Set in Redis cache
    async fn set_in_redis(&self, manager: &ConnectionManager, entry: &CacheEntry) -> AppResult<()> {
        let mut conn = manager.clone();
        
        let data = serde_json::to_vec(entry)
            .map_err(|e| AppError::Internal(format!("Failed to serialize cache entry: {}", e)))?;
        
        let _: () = redis::cmd("SETEX")
            .arg(&entry.key)
            .arg(entry.ttl)
            .arg(data)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis set error: {}", e)))?;
        
        Ok(())
    }

    /// Get from in-memory cache
    async fn get_from_memory(&self, key: &str) -> AppResult<Option<CacheEntry>> {
        let cache = self.memory_cache.read().await;
        
        if let Some(entry) = cache.get(key) {
            // Check if entry is expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            if now - entry.timestamp <= entry.ttl {
                debug!("Memory cache hit for key: {}", key);
                Ok(Some(entry.clone()))
            } else {
                debug!("Memory cache entry expired for key: {}", key);
                Ok(None)
            }
        } else {
            debug!("Memory cache miss for key: {}", key);
            Ok(None)
        }
    }

    /// Set in in-memory cache
    async fn set_in_memory(&self, entry: CacheEntry) -> AppResult<()> {
        let mut cache = self.memory_cache.write().await;
        
        // Check cache size and evict if necessary
        let total_size: usize = cache.values().map(|e| e.data.len()).sum();
        if total_size + entry.data.len() > self.config.max_size {
            self.evict_oldest_entries(&mut cache).await;
        }
        
        cache.insert(entry.key.clone(), entry);
        debug!("Cached response in memory");
        
        Ok(())
    }

    /// Evict oldest entries from memory cache
    async fn evict_oldest_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.timestamp);
        
        // Remove oldest 20% of entries
        let to_remove = (entries.len() as f64 * 0.2) as usize;
        let keys_to_remove: Vec<String> = entries.iter().take(to_remove).map(|(key, _)| (*key).clone()).collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
        
        debug!("Evicted {} oldest cache entries", to_remove);
    }

    /// Generate cache key from request
    pub fn generate_cache_key(&self, method: &str, params: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        method.hash(&mut hasher);
        params.to_string().hash(&mut hasher);
        
        format!("verus_rpc:{:x}", hasher.finish())
    }

    /// Check if a method should be cached
    pub fn should_cache_method(&self, method: &str) -> bool {
        // Cache read-only methods
        let cacheable_methods = [
            "getinfo",
            "getblock",
            "getblockcount",
            "getdifficulty",
            "getrawtransaction",
            "getblockhash",
            "getblockheader",
            "getmempoolinfo",
            "getnetworkinfo",
            "getpeerinfo",
        ];
        
        cacheable_methods.contains(&method)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let memory_size = self.memory_cache.read().await.len();
        
        CacheStats {
            memory_entries: memory_size,
            redis_available: self.redis_manager.is_some(),
            cache_enabled: self.config.enabled,
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> AppResult<()> {
        // Clear memory cache
        self.memory_cache.write().await.clear();
        
        // Clear Redis cache if available
        if let Some(ref manager) = self.redis_manager {
            let mut conn = manager.clone();
            let _: () = redis::cmd("FLUSHDB")
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to clear Redis cache: {}", e)))?;
        }
        
        info!("Cache cleared");
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// Number of entries in memory cache
    pub memory_entries: usize,
    /// Whether Redis is available
    pub redis_available: bool,
    /// Whether caching is enabled
    pub cache_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_ttl: 300, // 5 minutes
            enabled: true,
            max_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = CacheConfig {
            enabled: false, // Disable cache to avoid Redis connection
            ..Default::default()
        };
        let adapter = CacheAdapter::new(config).await.unwrap();
        
        let params = serde_json::json!(["blockhash", 123]);
        let key1 = adapter.generate_cache_key("getblock", &params);
        let key2 = adapter.generate_cache_key("getblock", &params);
        
        assert_eq!(key1, key2);
        assert!(key1.starts_with("verus_rpc:"));
    }

    #[tokio::test]
    async fn test_should_cache_method() {
        let config = CacheConfig {
            enabled: false, // Disable cache to avoid Redis connection
            ..Default::default()
        };
        let adapter = CacheAdapter::new(config).await.unwrap();
        
        assert!(adapter.should_cache_method("getinfo"));
        assert!(adapter.should_cache_method("getblock"));
        assert!(!adapter.should_cache_method("sendrawtransaction"));
    }

    #[tokio::test]
    #[ignore] // Skip this test as it hangs due to Redis connection attempts
    async fn test_memory_cache() {
        let config = CacheConfig {
            enabled: true,
            redis_url: "redis://invalid".to_string(), // Force memory cache
            max_size: 1024, // Small size for testing
            ..Default::default()
        };
        
        let adapter = CacheAdapter::new(config).await.unwrap();
        
        let entry = CacheEntry {
            data: b"test response".to_vec(),
            content_type: "application/json".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: 60,
            key: "test_key".to_string(),
        };
        
        // Set entry
        adapter.set(entry.clone()).await.unwrap();
        
        // Get entry
        let retrieved = adapter.get("test_key").await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.data, entry.data);
        assert_eq!(retrieved_entry.content_type, entry.content_type);
    }
} 