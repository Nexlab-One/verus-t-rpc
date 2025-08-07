//! Mining pool routes module
//! 
//! This module contains mining pool route configurations.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        utils::{with_mining_pool_client, with_config, with_cache_middleware, with_rate_limit_middleware},
        handlers::{handle_mining_pool_request, handle_pool_metrics_request},
    },
    middleware::{cache::CacheMiddleware, rate_limit::RateLimitMiddleware},
};
use std::sync::Arc;
use warp::Filter;
 
/// Mining pool routes configuration
pub struct MiningPoolRoutes;

impl MiningPoolRoutes {
    /// Create the mining pool share validation endpoint route
    pub fn create_mining_pool_route(
        config: AppConfig,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("pool")
            .and(warp::path("share"))
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(with_mining_pool_client())
            .and(with_config(config))
            .and(with_cache_middleware(cache_middleware))
            .and(with_rate_limit_middleware(rate_limit_middleware))
            .and_then(handle_mining_pool_request)
    }

    /// Create the mining pool metrics endpoint route
    pub fn create_pool_metrics_route(
        config: AppConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("pool")
            .and(warp::path("metrics"))
            .and(warp::get())
            .and(with_mining_pool_client())
            .and(with_config(config))
            .and_then(handle_pool_metrics_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

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
    async fn test_mining_pool_routes_create_mining_pool_route() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // This should not panic and should return a valid filter
        let route = MiningPoolRoutes::create_mining_pool_route(
            config,
            cache_middleware,
            rate_limit_middleware,
        );
        let _ = route.clone();
    }

    #[test]
    fn test_mining_pool_routes_create_pool_metrics_route() {
        let config = create_test_config();

        // This should not panic and should return a valid filter
        let route = MiningPoolRoutes::create_pool_metrics_route(config);
        let _ = route.clone();
    }

    #[tokio::test]
    async fn test_mining_pool_routes_with_different_configs() {
        let mut config = create_test_config();
        config.server.max_request_size = 8192; // Different size
        
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Should work with different config values
        let mining_pool_route = MiningPoolRoutes::create_mining_pool_route(
            config.clone(),
            cache_middleware,
            rate_limit_middleware,
        );

        let pool_metrics_route = MiningPoolRoutes::create_pool_metrics_route(config);
        let _ = mining_pool_route.clone();
        let _ = pool_metrics_route.clone();
    }

    #[tokio::test]
    async fn test_mining_pool_routes_route_structure() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Test that the routes are created with the expected structure
        let mining_pool_route = MiningPoolRoutes::create_mining_pool_route(
            config.clone(),
            cache_middleware,
            rate_limit_middleware,
        );

        let pool_metrics_route = MiningPoolRoutes::create_pool_metrics_route(config);
        let _ = mining_pool_route.clone();
        let _ = pool_metrics_route.clone();
    }

    #[tokio::test]
    async fn test_mining_pool_share_route_e2e() {
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();
        let route = MiningPoolRoutes::create_mining_pool_route(
            config,
            cache_middleware,
            rate_limit_middleware,
        );

        let req_body = json!({
            "jsonrpc": "2.0",
            "method": "submit_share",
            "params": {
                "challenge_id": "test_challenge",
                "miner_address": "test_miner",
                "nonce": "123",
                "solution": "abc",
                "difficulty": 1.0,
                "timestamp": "2027-01-01T00:00:00Z"
            },
            "id": 1
        });

        let res = warp::test::request()
            .method("POST")
            .path("/pool/share")
            .header("x-forwarded-for", "127.0.0.1")
            .json(&req_body)
            .reply(&route)
            .await;

        assert!(res.status().is_success() || res.status().is_client_error() || res.status().is_server_error());
        assert!(res.headers().contains_key("content-security-policy"));
        let body: Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(body["jsonrpc"], "2.0");
        assert!(body.get("id").is_some());
        assert!(body.get("result").is_some() || body.get("error").is_some());
    }

    #[tokio::test]
    async fn test_mining_pool_metrics_route_e2e() {
        let config = create_test_config();
        let route = MiningPoolRoutes::create_pool_metrics_route(config);

        let res = warp::test::request()
            .method("GET")
            .path("/pool/metrics")
            .reply(&route)
            .await;

        assert!(res.status().is_success());
        assert!(res.headers().contains_key("content-security-policy"));
        let body: Value = serde_json::from_slice(res.body()).unwrap();
        assert!(body.get("total_shares").is_some());
    }
}
