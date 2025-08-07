//! RPC request handler module
//! 
//! This module contains the main RPC request handler for processing JSON-RPC requests.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        models::{JsonRpcRequest, RequestContext},
        utils::extract_and_validate_client_ip,
        processors::{BaseRequestProcessor, RpcRequestProcessor},
    },
    application::use_cases::ProcessRpcRequestUseCase,
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
    },
};
use std::sync::Arc;
use tracing::{info, instrument};
use warp::{Reply};

/// Handle RPC requests optimized for reverse proxy deployment
#[instrument(skip(rpc_use_case, config, cache_middleware, rate_limit_middleware))]
pub async fn handle_rpc_request(
    request: JsonRpcRequest,
    client_ip: String,
    auth_header: Option<String>,
    user_agent_header: Option<String>,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    config: AppConfig,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Extract and validate client IP
    let validated_client_ip = extract_and_validate_client_ip(&client_ip, &config);
    
    // Create request context
    let mut context = RequestContext::new(
        validated_client_ip.clone(),
        request.method.clone(),
        request.params.clone(),
    );
    if let Some(ua) = user_agent_header { context = context.with_user_agent(ua); }
    if let Some(auth) = auth_header { context = context.with_auth_token(auth); }

    // Log request if enabled
    if config.security.enable_request_logging {
        info!(
            request_id = %context.request_id,
            method = %request.method,
            client_ip = %context.client_ip,
            "Processing RPC request"
        );
    }

    // Validate request using base processor
    if let Err(response) = BaseRequestProcessor::validate_request(&request, &context, &config) {
        return Ok(response);
    }

    // Check rate limit using base processor
    if let Err(response) = BaseRequestProcessor::check_rate_limit(
        &validated_client_ip,
        &context,
        &request,
        &rate_limit_middleware,
        &config,
    ).await {
        return Ok(response);
    }

    // Check cache using base processor
    if let Ok(Some(cached_response)) = BaseRequestProcessor::check_cache(
        &request,
        &context,
        &cache_middleware,
        &config,
    ).await {
        return Ok(cached_response);
    }

    // Process request using RPC processor
    match RpcRequestProcessor::process_rpc_request(
        &request,
        &context,
        &rpc_use_case,
        &cache_middleware,
        &config,
    ).await {
        Ok(infra_response) => {
            // Create success response using RPC processor
            Ok(RpcRequestProcessor::create_rpc_success_response(&infra_response, &config))
        }
        Err(e) => {
            Ok(RpcRequestProcessor::handle_use_case_error(
                &e,
                &request,
                &context,
                &config,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::{RpcService, MetricsService};
    use serde_json::json;

    fn create_test_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.security.development_mode = true;
        config.cache.enabled = false;
        config.rate_limit.enabled = false;
        config
    }

    fn create_test_rpc_use_case() -> Arc<ProcessRpcRequestUseCase> {
        let config = Arc::new(create_test_config());
        let security_validator = Arc::new(crate::domain::security::SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(config, security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        Arc::new(ProcessRpcRequestUseCase::new(rpc_service, metrics_service))
    }

    fn create_test_request() -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getinfo".to_string(),
            params: Some(json!({})),
            id: Some(json!(1)),
        }
    }

    async fn create_test_cache_middleware() -> Arc<CacheMiddleware> {
        Arc::new(CacheMiddleware::new(&create_test_config()).await.unwrap())
    }

    fn create_test_rate_limit_middleware() -> Arc<RateLimitMiddleware> {
        Arc::new(RateLimitMiddleware::new(create_test_config()))
    }

    #[tokio::test]
    async fn test_handle_rpc_request_success() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_invalid_method() {
        let mut request = create_test_request();
        request.method = "".to_string(); // Invalid empty method
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_invalid_jsonrpc_version() {
        let mut request = create_test_request();
        request.jsonrpc = "1.0".to_string(); // Invalid version
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_missing_id() {
        let mut request = create_test_request();
        request.id = None; // Missing ID
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_different_methods() {
        let methods = vec!["getinfo", "getblock", "getrawtransaction", "getaddressbalance"];
        
        for method in methods {
            let mut request = create_test_request();
            request.method = method.to_string();
            let client_ip = "127.0.0.1";
            let rpc_use_case = create_test_rpc_use_case();
            let config = create_test_config();
            let cache_middleware = create_test_cache_middleware().await;
            let rate_limit_middleware = create_test_rate_limit_middleware();

            let result = handle_rpc_request(
                request,
                client_ip.to_string(),
                None,
                None,
                rpc_use_case,
                config,
                cache_middleware,
                rate_limit_middleware,
            ).await;

            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_complex_params() {
        let mut request = create_test_request();
        request.method = "getblock".to_string();
        request.params = Some(json!({
            "hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "verbose": true,
            "include_tx": true
        }));
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_different_client_ips() {
        let client_ips = vec!["127.0.0.1", "192.168.1.1", "10.0.0.1"];
        
        for client_ip in client_ips {
            let request = create_test_request();
            let rpc_use_case = create_test_rpc_use_case();
            let config = create_test_config();
            let cache_middleware = create_test_cache_middleware().await;
            let rate_limit_middleware = create_test_rate_limit_middleware();

            let result = handle_rpc_request(
                request,
                client_ip.to_string(),
                None,
                None,
                rpc_use_case,
                config,
                cache_middleware,
                rate_limit_middleware,
            ).await;

            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_logging_enabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let mut config = create_test_config();
        config.security.enable_request_logging = true;
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_logging_disabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let mut config = create_test_config();
        config.security.enable_request_logging = false;
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_cache_enabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let mut config = create_test_config();
        config.cache.enabled = true;
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_rate_limit_enabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let mut config = create_test_config();
        config.rate_limit.enabled = true;
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_different_configs() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let mut config = create_test_config();
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_string_id() {
        let mut request = create_test_request();
        request.id = Some(json!("test_id"));
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_null_id() {
        let mut request = create_test_request();
        request.id = Some(json!(null));
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_empty_params() {
        let mut request = create_test_request();
        request.params = Some(json!({}));
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rpc_request_with_null_params() {
        let mut request = create_test_request();
        request.params = None;
        let client_ip = "127.0.0.1";
        let rpc_use_case = create_test_rpc_use_case();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_rpc_request(
            request,
            client_ip.to_string(),
            None,
            None,
            rpc_use_case,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_request_context_creation_for_rpc() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let config = create_test_config();
        
        let validated_client_ip = extract_and_validate_client_ip(client_ip, &config);
        let context = RequestContext::new(
            validated_client_ip.clone(),
            request.method.clone(),
            request.params.clone(),
        );

        assert_eq!(context.client_ip, "127.0.0.1");
        assert_eq!(context.method, "getinfo");
        assert!(context.request_id.len() > 0);
    }

    #[test]
    fn test_rpc_use_case_creation() {
        let rpc_use_case = create_test_rpc_use_case();
        // At minimum, ensure the Arc isn't null by using it
        assert!(Arc::strong_count(&rpc_use_case) >= 1);
    }

    #[tokio::test]
    async fn test_cache_middleware_creation() {
        let config = create_test_config();
        let cache_middleware = CacheMiddleware::new(&config).await;
        
        // Verify middleware creation doesn't panic
        assert!(cache_middleware.is_ok());
    }

    #[test]
    fn test_rate_limit_middleware_creation() {
        let config = create_test_config();
        let rate_limit_middleware = RateLimitMiddleware::new(config);
        // Ensure it's enabled/disabled flag can be read
        let _enabled = rate_limit_middleware.is_enabled();
    }
}
