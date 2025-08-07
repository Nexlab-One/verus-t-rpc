//! Base request processor module
//! 
//! This module contains the base request processing patterns that are common
//! across all endpoint handlers.

use crate::{
    config::AppConfig,
    infrastructure::http::{
        models::{JsonRpcRequest, JsonRpcResponse, RequestContext},
        utils::extract_and_validate_client_ip,
    },
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
        security_headers::{SecurityHeadersMiddleware, create_json_response_with_security_headers},
    },
};
use std::sync::Arc;
use tracing::{error, info, warn, debug};

/// Base request processor that handles common processing patterns
pub struct BaseRequestProcessor;

impl BaseRequestProcessor {
    /// Extract and validate client IP, create request context, and log request
    pub fn setup_request_context(
        request: &JsonRpcRequest,
        client_ip: &str,
        config: &AppConfig,
    ) -> (String, RequestContext) {
        let validated_client_ip = extract_and_validate_client_ip(client_ip, config);
        
        let context = RequestContext::new(
            validated_client_ip.clone(),
            request.method.clone(),
            request.params.clone(),
        );

        // Log request if enabled
        if config.security.enable_request_logging {
            info!(
                request_id = %context.request_id,
                method = %request.method,
                client_ip = %context.client_ip,
                "Processing request"
            );
        }

        (validated_client_ip, context)
    }

    /// Validate the request and return error response if validation fails
    pub fn validate_request(
        request: &JsonRpcRequest,
        context: &RequestContext,
        config: &AppConfig,
    ) -> Result<(), warp::reply::WithStatus<Box<dyn warp::Reply>>> {
        if let Err(e) = request.validate_request() {
            error!(
                request_id = %context.request_id,
                error = %e,
                "Request validation failed"
            );
            return Err(Self::create_error_response_with_security_headers(
                "Invalid request",
                &request.id,
                warp::http::StatusCode::BAD_REQUEST,
                config,
            ));
        }
        Ok(())
    }

    /// Check rate limit and return error response if rate limit is exceeded
    pub async fn check_rate_limit(
        client_ip: &str,
        context: &RequestContext,
        request: &JsonRpcRequest,
        rate_limit_middleware: &Arc<RateLimitMiddleware>,
        config: &AppConfig,
    ) -> Result<(), warp::reply::WithStatus<Box<dyn warp::Reply>>> {
        if rate_limit_middleware.is_enabled() {
            let client_limiter = rate_limit_middleware.create_client_limiter(client_ip);
            if let Err(e) = client_limiter.check_rate_limit(client_ip).await {
                error!(
                    request_id = %context.request_id,
                    client_ip = %client_ip,
                    error = %e,
                    "Rate limit exceeded"
                );
                let error_response = JsonRpcResponse::error(
                    crate::infrastructure::http::models::JsonRpcError::internal_error("Rate limit exceeded"),
                    request.id.clone(),
                );
                
                let security_middleware = SecurityHeadersMiddleware::new(config.clone());
                let response = create_json_response_with_security_headers(
                    &error_response,
                    &security_middleware,
                );
                
                return Err(warp::reply::with_status(
                    response,
                    warp::http::StatusCode::TOO_MANY_REQUESTS,
                ));
            }
        }
        Ok(())
    }

    /// Check cache for read-only methods and return cached response if available
    pub async fn check_cache(
        request: &JsonRpcRequest,
        context: &RequestContext,
        cache_middleware: &Arc<CacheMiddleware>,
        config: &AppConfig,
    ) -> Result<Option<warp::reply::WithStatus<Box<dyn warp::Reply>>>, ()> {
        if cache_middleware.should_cache_response(&request.method, 200) {
            let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
            let cache_key = cache_middleware.generate_cache_key(&request.method, params);
            
            if let Ok(Some(cached_entry)) = cache_middleware.get_cached_response(&cache_key).await {
                info!(
                    request_id = %context.request_id,
                    method = %request.method,
                    "Cache hit - returning cached response"
                );
                
                // Return cached response as JSON with security headers
                let cached_response: JsonRpcResponse = serde_json::from_slice(&cached_entry.data)
                    .unwrap_or_else(|_| JsonRpcResponse::error(
                        crate::infrastructure::http::models::JsonRpcError::internal_error("Failed to deserialize cached response"),
                        request.id.clone(),
                    ));
                
                let security_middleware = SecurityHeadersMiddleware::new(config.clone());
                let response = create_json_response_with_security_headers(
                    &cached_response,
                    &security_middleware,
                );
                
                return Ok(Some(warp::reply::with_status(
                    response,
                    warp::http::StatusCode::OK,
                )));
            }
        }
        Ok(None)
    }

    /// Create error response with security headers - common pattern used across handlers
    pub fn create_error_response_with_security_headers(
        error_message: &str,
        request_id: &Option<serde_json::Value>,
        status_code: warp::http::StatusCode,
        config: &AppConfig,
    ) -> warp::reply::WithStatus<Box<dyn warp::Reply>> {
        let error_response = JsonRpcResponse::error(
            crate::infrastructure::http::models::JsonRpcError::internal_error(error_message),
            request_id.clone(),
        );
        
        let security_middleware = SecurityHeadersMiddleware::new(config.clone());
        let response = create_json_response_with_security_headers(
            &error_response,
            &security_middleware,
        );
        
        warp::reply::with_status(response, status_code)
    }

    /// Cache response if it's a cacheable method
    pub async fn cache_response(
        request: &JsonRpcRequest,
        context: &RequestContext,
        response: &JsonRpcResponse,
        cache_middleware: &Arc<CacheMiddleware>,
        config: &AppConfig,
    ) {
        if cache_middleware.should_cache_response(&request.method, 200) {
            let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
            let cache_key = cache_middleware.generate_cache_key(&request.method, params);
            
            // Serialize response for caching
            if let Ok(response_data) = serde_json::to_vec(response) {
                let cache_entry = cache_middleware.create_cache_entry(
                    cache_key,
                    response_data,
                    "application/json".to_string(),
                    config.cache.default_ttl,
                );
                
                // Cache the response (fire and forget)
                if let Err(e) = cache_middleware.cache_response(cache_entry).await {
                    warn!(
                        request_id = %context.request_id,
                        error = %e,
                        "Failed to cache response"
                    );
                } else {
                    debug!(
                        request_id = %context.request_id,
                        method = %request.method,
                        "Response cached successfully"
                    );
                }
            }
        }
    }

    /// Create success response with security headers
    pub fn create_success_response(
        response: &JsonRpcResponse,
        config: &AppConfig,
    ) -> warp::reply::WithStatus<Box<dyn warp::Reply>> {
        let security_middleware = SecurityHeadersMiddleware::new(config.clone());
        let response_with_headers = create_json_response_with_security_headers(
            response,
            &security_middleware,
        );
        
        warp::reply::with_status(
            response_with_headers,
            warp::http::StatusCode::OK,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::models::{JsonRpcRequest, JsonRpcResponse};
    use warp::Reply;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_request() -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getinfo".to_string(),
            params: Some(serde_json::json!([])),
            id: Some(serde_json::json!(1)),
        }
    }

    fn create_test_rate_limit_middleware() -> Arc<RateLimitMiddleware> {
        Arc::new(RateLimitMiddleware::new(create_test_config()))
    }

    async fn create_test_cache_middleware() -> Arc<CacheMiddleware> {
        Arc::new(CacheMiddleware::new(&create_test_config()).await.unwrap())
    }

    #[test]
    fn test_setup_request_context() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let config = create_test_config();

        let (validated_ip, context) = BaseRequestProcessor::setup_request_context(
            &request,
            client_ip,
            &config,
        );

        assert_eq!(validated_ip, "127.0.0.1");
        assert_eq!(context.client_ip, "127.0.0.1");
        assert_eq!(context.method, "getinfo");
        assert!(context.request_id.len() > 0);
    }

    #[test]
    fn test_setup_request_context_with_proxy_ip() {
        let request = create_test_request();
        let client_ip = "192.168.1.100"; // Single IP, not comma-separated
        let mut config = create_test_config();
        
        // Add X-Forwarded-For to trusted proxy headers to test proxy behavior
        config.security.trusted_proxy_headers.push("X-Forwarded-For".to_string());

        let (validated_ip, context) = BaseRequestProcessor::setup_request_context(
            &request,
            client_ip,
            &config,
        );

        assert_eq!(validated_ip, "192.168.1.100"); // Should return the IP as-is when proxy headers are trusted
        assert_eq!(context.client_ip, "192.168.1.100");
    }

    #[test]
    fn test_validate_request_success() {
        let request = create_test_request();
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        );
        let config = create_test_config();

        let result = BaseRequestProcessor::validate_request(&request, &context, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request_invalid_method() {
        let mut request = create_test_request();
        request.method = "".to_string(); // Empty method will fail validation
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        );
        let config = create_test_config();

        let result = BaseRequestProcessor::validate_request(&request, &context, &config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_rate_limit_disabled() {
        let request = create_test_request();
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        );
        let rate_limit_middleware = create_test_rate_limit_middleware();
        let config = create_test_config();

        let result = BaseRequestProcessor::check_rate_limit(
            "127.0.0.1",
            &context,
            &request,
            &rate_limit_middleware,
            &config,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_cache_disabled() {
        let request = create_test_request();
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        );
        let cache_middleware = create_test_cache_middleware().await;
        let config = create_test_config();

        let result = BaseRequestProcessor::check_cache(
            &request,
            &context,
            &cache_middleware,
            &config,
        ).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_create_error_response_with_security_headers() {
        let config = create_test_config();
        let request_id = Some(serde_json::json!(1));

        let reply = BaseRequestProcessor::create_error_response_with_security_headers(
            "Test error",
            &request_id,
            warp::http::StatusCode::BAD_REQUEST,
            &config,
        );
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
        let headers = response.headers();
        assert!(headers.get("content-security-policy").is_some());
    }

    #[test]
    fn test_create_success_response() {
        let config = create_test_config();
        let json_response = JsonRpcResponse::success(
            serde_json::json!({"result": "test"}),
            Some(serde_json::json!(1)),
        );
        let reply = BaseRequestProcessor::create_success_response(&json_response, &config);
        let response = reply.into_response();
        assert_eq!(response.status(), warp::http::StatusCode::OK);
        let headers = response.headers();
        assert!(headers.get("content-security-policy").is_some());
    }

    #[tokio::test]
    async fn test_cache_response() {
        let request = create_test_request();
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "getinfo".to_string(),
            Some(serde_json::json!([])),
        );
        let response = JsonRpcResponse::success(
            serde_json::json!({"result": "test"}),
            Some(serde_json::json!(1)),
        );
        let cache_middleware = create_test_cache_middleware().await;
        let config = create_test_config();

        // This should not panic
        BaseRequestProcessor::cache_response(
            &request,
            &context,
            &response,
            &cache_middleware,
            &config,
        ).await;
    }

    #[test]
    fn test_setup_request_context_with_logging_disabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let mut config = create_test_config();
        config.security.enable_request_logging = false;

        let (validated_ip, context) = BaseRequestProcessor::setup_request_context(
            &request,
            client_ip,
            &config,
        );

        assert_eq!(validated_ip, "127.0.0.1");
        assert_eq!(context.client_ip, "127.0.0.1");
        assert_eq!(context.method, "getinfo");
    }

    #[test]
    fn test_validate_request_empty_method() {
        let mut request = create_test_request();
        request.method = "".to_string();
        let context = RequestContext::new(
            "127.0.0.1".to_string(),
            "".to_string(),
            Some(serde_json::json!([])),
        );
        let config = create_test_config();

        let result = BaseRequestProcessor::validate_request(&request, &context, &config);
        assert!(result.is_err());
    }
}
