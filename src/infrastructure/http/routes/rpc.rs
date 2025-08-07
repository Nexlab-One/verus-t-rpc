//! RPC routes module
//! 
//! This module contains RPC-specific route configurations.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        utils::{with_rpc_use_case, with_config, with_cache_middleware, with_rate_limit_middleware},
        handlers::handle_rpc_request,
    },
    application::use_cases::ProcessRpcRequestUseCase,
    middleware::{cache::CacheMiddleware, rate_limit::RateLimitMiddleware},
};
use std::sync::Arc;
use warp::Filter;

/// RPC routes configuration
pub struct RpcRoutes;

impl RpcRoutes {
    /// Create the main RPC endpoint route
    pub fn create_rpc_route(
        config: AppConfig,
        rpc_use_case: Arc<ProcessRpcRequestUseCase>,
        cache_middleware: Arc<CacheMiddleware>,
        rate_limit_middleware: Arc<RateLimitMiddleware>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path::end()
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(warp::header::optional::<String>("authorization"))
            .and(warp::header::optional::<String>("user-agent"))
            .and(with_rpc_use_case(rpc_use_case))
            .and(with_config(config))
            .and(with_cache_middleware(cache_middleware))
            .and(with_rate_limit_middleware(rate_limit_middleware))
            .and_then(handle_rpc_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::use_cases::ProcessRpcRequestUseCase;
    use crate::application::services::{RpcService, MetricsService};
    use crate::domain::security::SecurityPolicy;
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

    fn create_test_rpc_use_case() -> Arc<ProcessRpcRequestUseCase> {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(crate::domain::security::SecurityValidator::new(SecurityPolicy::default()));
        let rpc_service = Arc::new(RpcService::new(config.clone(), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(ProcessRpcRequestUseCase::new(rpc_service, metrics_service))
    }

    #[tokio::test]
    async fn test_rpc_routes_creation() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let route = RpcRoutes::create_rpc_route(
            config,
            rpc_use_case,
            cache_middleware,
            rate_limit_middleware,
        );
        let _ = route.clone();
    }

    #[tokio::test]
    async fn test_rpc_routes_with_different_configs() {
        let mut config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        // Test with different configurations
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let route = RpcRoutes::create_rpc_route(
            config,
            rpc_use_case,
            cache_middleware,
            rate_limit_middleware,
        );
        let _ = route.clone();
    }

    #[tokio::test]
    async fn test_rpc_route_e2e_valid_request() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let route = RpcRoutes::create_rpc_route(
            config,
            rpc_use_case,
            cache_middleware,
            rate_limit_middleware,
        );

        let req_body = json!({
            "jsonrpc": "2.0",
            "method": "getinfo",
            "params": {},
            "id": 1
        });

        let res = warp::test::request()
            .method("POST")
            .path("/")
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
    async fn test_rpc_route_e2e_invalid_method() {
        let config = create_test_config();
        let rpc_use_case = create_test_rpc_use_case();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let route = RpcRoutes::create_rpc_route(
            config,
            rpc_use_case,
            cache_middleware,
            rate_limit_middleware,
        );

        let req_body = json!({
            "jsonrpc": "2.0",
            "method": "",
            "params": {},
            "id": 1
        });

        let res = warp::test::request()
            .method("POST")
            .path("/")
            .header("x-forwarded-for", "127.0.0.1")
            .json(&req_body)
            .reply(&route)
            .await;

        assert!(res.status().is_client_error() || res.status().is_server_error());
        assert!(res.headers().contains_key("content-security-policy"));
        let body: Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(body["jsonrpc"], "2.0");
        assert!(body.get("error").is_some());
        assert!(body.get("id").is_some());
    }
}
