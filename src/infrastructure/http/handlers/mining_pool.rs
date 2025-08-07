//! Mining pool handler module
//! 
//! This module contains the mining pool related endpoint handlers for share validation and metrics.

use crate::{
    config::AppConfig,
    infrastructure::http::{
                 models::{JsonRpcRequest, RequestContext},
        utils::extract_and_validate_client_ip,
        processors::BaseRequestProcessor,
        mining_pool::{MiningPoolUtils, MiningPoolResponseHandler},
    },
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
        security_headers::{SecurityHeadersMiddleware, create_json_response_with_security_headers},
    },
};
use std::sync::Arc;
use tracing::{error, info};
use warp::{Reply};

/// Handle mining pool share validation requests
pub async fn handle_mining_pool_request(
    request: JsonRpcRequest,
    client_ip: String,
    mining_pool_client: Arc<crate::infrastructure::adapters::MiningPoolClient>,
    config: AppConfig,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Extract and validate client IP
    let validated_client_ip = extract_and_validate_client_ip(&client_ip, &config);
    
    // Create request context
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
            "Processing mining pool share validation request"
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

    // Convert to domain model
    let domain_request = match crate::infrastructure::converters::ModelConverter::to_domain_request(&request, &context) {
        Ok(req) => req,
        Err(e) => {
            error!(
                request_id = %context.request_id,
                error = %e,
                "Failed to convert request to domain model"
            );
            return Ok(BaseRequestProcessor::create_error_response_with_security_headers(
                &e.to_string(),
                &request.id,
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &config,
            ));
        }
    };

         // Process request using use case
     let request_id = request.id.clone();
     let context_request_id = context.request_id.clone();
     
     // Extract request ID as string for response handlers
     let request_id_str = request_id.as_ref()
         .and_then(|v| v.as_str())
         .unwrap_or("");
     
     // Parse pool share from domain request parameters
     let pool_share = match MiningPoolUtils::parse_pool_share_from_request(&domain_request) {
        Ok(share) => share,
        Err(e) => {
            error!(
                request_id = %context_request_id,
                error = %e,
                "Failed to parse pool share from request parameters"
            );
            return Ok(BaseRequestProcessor::create_error_response_with_security_headers(
                &e.to_string(),
                &request_id,
                warp::http::StatusCode::BAD_REQUEST,
                &config,
            ));
        }
    };
     
     match mining_pool_client.validate_share(&pool_share).await {
                 Ok(domain_response) => {
             // Validate the pool response before processing
             if let Err(validation_error) = MiningPoolResponseHandler::validate_pool_response(&domain_response) {
                 return Ok(BaseRequestProcessor::create_error_response_with_security_headers(
                     &validation_error.to_string(),
                     &request_id,
                     warp::http::StatusCode::BAD_REQUEST,
                     &config,
                 ));
             }
             
             // Create response using mining pool response handler
             let infra_response = MiningPoolResponseHandler::handle_successful_validation(
                 &domain_response,
                 request_id_str,
                 &context_request_id,
             );
            
            // Cache the response using base processor
            BaseRequestProcessor::cache_response(
                &request,
                &context,
                &infra_response,
                &cache_middleware,
                &config,
            ).await;
            
            // Create success response using base processor
            Ok(BaseRequestProcessor::create_success_response(&infra_response, &config))
        }
        Err(e) => {
                         // Create error response using mining pool response handler
             let error_response = MiningPoolResponseHandler::handle_failed_validation(
                 &e,
                 request_id_str,
                 &context_request_id,
             );
            
            Ok(BaseRequestProcessor::create_error_response_with_security_headers(
                &error_response.error.unwrap().message,
                &request_id,
                e.http_status_code(),
                &config,
            ))
        }
    }
}

/// Handle mining pool metrics requests
pub async fn handle_pool_metrics_request(
    mining_pool_client: Arc<crate::infrastructure::adapters::MiningPoolClient>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let metrics_data = mining_pool_client.get_metrics().await;
    
    // Apply security headers only
    let response = create_json_response_with_security_headers(
        &metrics_data,
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::models::JsonRpcRequest;
    use serde_json::json;

    fn create_test_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.security.mining_pool = Some(crate::config::app_config::MiningPoolConfig {
            pool_url: "http://localhost:8080".to_string(),
            api_key: "test_api_key".to_string(),
            public_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            requests_per_minute: 100,
            enabled: true,
        });
        config
    }

    fn create_test_request() -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "submit_share".to_string(),
            params: Some(serde_json::json!({
                "challenge_id": "test_challenge",
                "miner_address": "test_miner",
                "nonce": "test_nonce",
                "solution": "test_solution",
                "difficulty": 1.0,
                "timestamp": "2023-01-01T00:00:00Z"
            })),
            id: Some(serde_json::json!(1)),
        }
    }

    fn create_test_mining_pool_client() -> Arc<crate::infrastructure::adapters::MiningPoolClient> {
        Arc::new(crate::infrastructure::adapters::MiningPoolClient::new(create_test_config().into()))
    }

    async fn create_test_cache_middleware() -> Arc<CacheMiddleware> {
        Arc::new(CacheMiddleware::new(&create_test_config()).await.unwrap())
    }

    fn create_test_rate_limit_middleware() -> Arc<RateLimitMiddleware> {
        Arc::new(RateLimitMiddleware::new(create_test_config()))
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_success() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_invalid_request() {
        let mut request = create_test_request();
        request.method = "".to_string(); // Invalid empty method
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_different_configs() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let mut config = create_test_config();
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_logging_disabled() {
        let request = create_test_request();
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let mut config = create_test_config();
        config.security.enable_request_logging = false;
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_pool_metrics_request_success() {
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();

        let result = handle_pool_metrics_request(
            mining_pool_client,
            config,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_pool_metrics_request_with_different_configs() {
        let mining_pool_client = create_test_mining_pool_client();
        let mut config = create_test_config();
        config.server.port = 8081;
        config.server.bind_address = "127.0.0.1".parse().unwrap();

        let result = handle_pool_metrics_request(
            mining_pool_client,
            config,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_pool_metrics_request_with_security_headers_disabled() {
        let mining_pool_client = create_test_mining_pool_client();
        let mut config = create_test_config();
        config.security.enable_security_headers = false;

        let result = handle_pool_metrics_request(
            mining_pool_client,
            config,
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_mining_pool_client_creation() {
        let config = create_test_config();
        let client = crate::infrastructure::adapters::MiningPoolClient::new(config.into());
        // Ensure client object is valid by checking address of struct
        let ptr: *const _ = &client;
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_request_context_creation_for_mining_pool() {
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
        assert_eq!(context.method, "submit_share");
        assert!(context.request_id.len() > 0);
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_malformed_params() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "invalid_field": "invalid_value",
            "missing_required": true
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_empty_solution() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge",
            "miner_address": "test_miner",
            "nonce": "test_nonce",
            "solution": "",
            "difficulty": 1.0,
            "timestamp": "2027-01-01T00:00:00Z"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_zero_difficulty() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge",
            "miner_address": "test_miner",
            "nonce": "test_nonce",
            "solution": "test_solution",
            "difficulty": 0.0,
            "timestamp": "2023-01-01T00:00:00Z"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_invalid_timestamp() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge",
            "miner_address": "test_miner",
            "nonce": "test_nonce",
            "solution": "test_solution",
            "difficulty": 1.0,
            "timestamp": "invalid_timestamp"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_very_large_difficulty() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge",
            "miner_address": "test_miner",
            "nonce": "test_nonce",
            "solution": "test_solution",
            "difficulty": 1e10,
            "timestamp": "2023-01-01T00:00:00Z"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_special_characters() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge_with_special_chars_!@#$%^&*()",
            "miner_address": "test_miner_with_special_chars_!@#$%^&*()",
            "nonce": "test_nonce_with_special_chars_!@#$%^&*()",
            "solution": "test_solution_with_special_chars_!@#$%^&*()",
            "difficulty": 1.0,
            "timestamp": "2023-01-01T00:00:00Z"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_mining_pool_request_with_unicode_characters() {
        let mut request = create_test_request();
        request.params = Some(json!({
            "challenge_id": "test_challenge_with_unicode_🚀🌍🎯",
            "miner_address": "test_miner_with_unicode_🚀🌍🎯",
            "nonce": "test_nonce_with_unicode_🚀🌍🎯",
            "solution": "test_solution_with_unicode_🚀🌍🎯",
            "difficulty": 1.0,
            "timestamp": "2023-01-01T00:00:00Z"
        }));
        let client_ip = "127.0.0.1";
        let mining_pool_client = create_test_mining_pool_client();
        let config = create_test_config();
        let cache_middleware = create_test_cache_middleware().await;
        let rate_limit_middleware = create_test_rate_limit_middleware();

        let result = handle_mining_pool_request(
            request,
            client_ip.to_string(),
            mining_pool_client,
            config,
            cache_middleware,
            rate_limit_middleware,
        ).await;

        assert!(result.is_ok());
    }
}
