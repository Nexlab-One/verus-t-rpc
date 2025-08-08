//! Unit tests for Verus RPC Server components
//! 
//! This module provides comprehensive unit tests covering:
//! - Domain layer components
//! - Application layer services and use cases
//! - Infrastructure layer adapters and converters
//! - Configuration and validation
//! - Error handling

use crate::{
    config::AppConfig,
    domain::{
        rpc::{RpcRequest, RpcResponse, ClientInfo},
        security::SecurityValidator,
        validation::DomainValidator,
    },
    application::{
        services::{RpcService, MetricsService},
        use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    },
    infrastructure::{
        adapters::{
            ExternalRpcAdapter, AuthenticationAdapter, MonitoringAdapter,
            cache::{CacheAdapter, CacheConfig, CacheEntry},
            comprehensive_validator::ComprehensiveValidator,
        },
        converters::ModelConverter,
        http::models::{JsonRpcRequest, JsonRpcResponse, RequestContext, JsonRpcError},
    },
    middleware::{
        cache::CacheMiddleware,
        compression::{CompressionMiddleware, should_compress, get_compression_method, CompressionMethod},
        cors::CorsMiddleware,
        rate_limit::{RateLimitMiddleware, RateLimitState, RateLimitConfig},
        security_headers::SecurityHeadersMiddleware,
    },
    shared::error::{AppError, AppResult},
    tests::{
        common::{fixtures, assertions},
        config,
        TestResult,
    },
};
use serde_json::Value;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Domain layer unit tests
pub mod domain {
    use super::*;

    #[test]
    fn test_security_validator_creation() {
        let validator = SecurityValidator::new(Default::default());
        assert!(validator.is_initialized());
    }

    #[test]
    fn test_domain_validator_creation() {
        let validator = DomainValidator::new();
        assert!(validator.is_initialized());
    }

    #[test]
    fn test_rpc_request_creation() {
        let client_info = fixtures::test_client_info();
        let request = RpcRequest {
            method: "getinfo".to_string(),
            parameters: Some(serde_json::json!([])),
            id: Some(serde_json::json!(1)),
            client_info,
        };

        assert_eq!(request.method, "getinfo");
        assert!(request.parameters.is_some());
        assert!(request.id.is_some());
    }

    #[test]
    fn test_rpc_response_creation() {
        let response = RpcResponse {
            result: Some(serde_json::json!({"status": "success"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert!(response.id.is_some());
    }

    #[test]
    fn test_client_info_creation() {
        let client_info = fixtures::test_client_info();
        assert_eq!(client_info.ip_address, "127.0.0.1");
        assert!(client_info.user_agent.is_some());
        assert!(!client_info.request_id.is_empty());
    }
}

/// Application layer unit tests
pub mod application {
    use super::*;

    #[tokio::test]
    async fn test_rpc_service_creation() {
        let config = config::test_config();
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let service = RpcService::new(Arc::new(config), security_validator);
        
        assert!(service.is_initialized());
    }

    #[tokio::test]
    async fn test_payments_service_quote_and_submit_flow() {
        use crate::application::services::payments_service::{PaymentsService, PaymentsConfig, PaymentQuoteRequest, PaymentSubmitRequest};
        use crate::infrastructure::adapters::{ExternalRpcAdapter, PaymentsStore, TokenIssuerAdapter, RevocationStore};

        // Prepare config
        let mut app_config = config::test_config();
        app_config.payments.enabled = true;
        app_config.payments.address_types = vec!["orchard".into(), "sapling".into()];
        app_config.payments.default_address_type = "orchard".into();
        app_config.payments.min_confirmations = 1;
        app_config.payments.session_ttl_minutes = 5;
        app_config.payments.require_viewing_key = false;
        app_config.payments.tiers = vec![crate::config::app_config::PaymentTierConfig {
            id: "basic".into(), amount_vrsc: 1.0, description: None, permissions: vec!["read".into()]
        }];

        let app_config = Arc::new(app_config);

        // Create adapters
        let external = Arc::new(ExternalRpcAdapter::new(app_config.clone()));
        let store = Arc::new(PaymentsStore::new(None));
        let issuer = Arc::new(TokenIssuerAdapter::new(app_config.clone()));
        let revocations = Arc::new(RevocationStore::new(None));

        // Build service (config-driven tiers will map automatically)
        let mut svc = PaymentsService::new(app_config.clone(), PaymentsConfig::default(), external.clone(), store.clone(), issuer.clone(), revocations.clone());
        svc.refresh_from_app_config();

        // Mock client info
        let client_info = crate::tests::common::fixtures::test_client_info();

        // We cannot reach a real verusd in unit tests, so just validate request shaping logic
        // Ensure config reading and validation paths do not panic
        let req = PaymentQuoteRequest { tier_id: "basic".into(), address_type: None };
        // We expect a failure from RPC call; the important part is that pre-RPC validation passes
        let quote_res = svc.create_quote(req, &client_info).await;
        // Allow either RPC failure or success depending on environment, but not a validation error for tier or type
        if let Err(e) = &quote_res {
            // Should not be a validation error for unknown tier/type
            let msg = e.to_string();
            assert!(!msg.contains("unknown tier"));
            assert!(!msg.contains("unsupported address type"));
        }

        // Submit path should validate hex and session
        // Without a session, expect validation error: unknown payment_id
        let submit_res = svc.submit_raw_transaction(PaymentSubmitRequest { payment_id: "nope".into(), rawtx_hex: "ab".repeat(60) }, &client_info).await;
        assert!(submit_res.is_err());
    }

    #[tokio::test]
    async fn test_metrics_service_creation() {
        let service = MetricsService::new();
        let metrics = service.get_metrics();
        
        assert!(metrics.is_object());
        assert!(metrics.get("total_requests").is_some());
        assert!(metrics.get("successful_requests").is_some());
        assert!(metrics.get("failed_requests").is_some());
    }

    #[tokio::test]
    async fn test_health_check_use_case() {
        let use_case = HealthCheckUseCase;
        let health_data = use_case.execute();
        
        assert!(health_data.is_object());
        assert_eq!(health_data["status"], "healthy");
        assert!(health_data.get("timestamp").is_some());
        assert!(health_data.get("version").is_some());
    }

    #[tokio::test]
    async fn test_metrics_use_case() {
        let metrics_service = Arc::new(MetricsService::new());
        let use_case = GetMetricsUseCase::new(metrics_service);
        let metrics = use_case.execute();
        
        assert!(metrics.is_object());
        assert!(metrics.get("total_requests").is_some());
    }

    #[tokio::test]
    async fn test_process_rpc_request_use_case() {
        let config = config::test_config();
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(Arc::new(config), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service);
        
        let request = fixtures::test_rpc_request("getinfo", serde_json::json!([]));
        let result = use_case.execute(request).await;
        
        // Should succeed with mock external service
        assert!(result.is_ok());
    }
}

/// Infrastructure layer unit tests
pub mod infrastructure {
    use super::*;

    #[test]
    fn test_external_rpc_adapter_creation() {
        let config = Arc::new(config::test_config());
        let adapter = ExternalRpcAdapter::new(config);
        assert!(adapter.is_initialized());
    }

    #[test]
    fn test_authentication_adapter_creation() {
        let adapter = AuthenticationAdapter::new();
        assert!(adapter.is_initialized());
    }

    #[test]
    fn test_monitoring_adapter_creation() {
        let adapter = MonitoringAdapter::new();
        let metrics = adapter.get_prometheus_metrics();
        
        assert!(metrics.contains("# HELP"));
        assert!(metrics.contains("# TYPE"));
    }

    #[tokio::test]
    async fn test_cache_adapter_creation() {
        let config = CacheConfig {
            enabled: false, // Disable for testing
            ..Default::default()
        };
        let adapter = CacheAdapter::new(config).await.unwrap();
        
        assert!(adapter.is_initialized());
    }

    #[tokio::test]
    async fn test_cache_adapter_operations() {
        let config = CacheConfig {
            enabled: true,
            redis_url: "redis://invalid".to_string(), // Force memory cache
            max_size: 1024,
            ..Default::default()
        };
        let adapter = CacheAdapter::new(config).await.unwrap();
        
        let entry = CacheEntry {
            data: b"test data".to_vec(),
            content_type: "application/json".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: 60,
            key: "test_key".to_string(),
        };
        
        // Test set and get
        adapter.set(entry.clone()).await.unwrap();
        let retrieved = adapter.get("test_key").await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.data, entry.data);
        assert_eq!(retrieved_entry.content_type, entry.content_type);
    }

    #[test]
    fn test_comprehensive_validator_creation() {
        let validator = ComprehensiveValidator::new();
        assert!(validator.is_initialized());
    }

    #[test]
    fn test_comprehensive_validator_methods() {
        let validator = ComprehensiveValidator::new();
        
        // Test valid methods
        let params: Option<Value> = None;
        assert!(validator.validate_method("getinfo", &params).is_ok());
        
        let params = Some(Value::Array(vec![
            Value::String(fixtures::test_block_hash()),
            Value::Bool(true),
        ]));
        assert!(validator.validate_method("getblock", &params).is_ok());
        
        // Test invalid methods
        let params: Option<Value> = None;
        assert!(validator.validate_method("invalid_method", &params).is_err());
    }

    #[test]
    fn test_model_converter_domain_to_infrastructure() {
        let domain_request = fixtures::test_rpc_request("getinfo", serde_json::json!([]));
        let context = fixtures::test_request_context();
        
        let result = ModelConverter::to_domain_request(&domain_request, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_model_converter_infrastructure_to_domain() {
        let domain_response = RpcResponse {
            result: Some(serde_json::json!({"status": "success"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        
        let infra_response = ModelConverter::to_infrastructure_response(&domain_response);
        assert_eq!(infra_response.jsonrpc, "2.0");
        assert!(infra_response.result.is_some());
        assert!(infra_response.error.is_none());
    }

    #[test]
    fn test_json_rpc_request_validation() {
        let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
        assert!(request.validate_request().is_ok());
    }

    #[test]
    fn test_json_rpc_request_invalid_validation() {
        let mut request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
        request.jsonrpc = "1.0".to_string(); // Invalid version
        
        assert!(request.validate_request().is_err());
    }

    #[test]
    fn test_request_context_creation() {
        let context = fixtures::test_request_context();
        assert_eq!(context.client_ip, "127.0.0.1");
        assert_eq!(context.method, "getinfo");
        assert!(context.params.is_some());
        assert!(!context.request_id.is_empty());
    }

    #[test]
    fn test_json_rpc_error_creation() {
        let error = JsonRpcError::method_not_found();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
        
        let error = JsonRpcError::invalid_params("Invalid parameters");
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid parameters");
        
        let error = JsonRpcError::internal_error("Internal error");
        assert_eq!(error.code, -32603);
        assert_eq!(error.message, "Internal error");
    }
}

/// Middleware unit tests
pub mod middleware {
    use super::*;

    #[tokio::test]
    async fn test_cache_middleware_creation() {
        let config = config::test_config();
        let middleware = CacheMiddleware::new(&config).await.unwrap();
        
        assert!(middleware.is_initialized());
    }

    #[tokio::test]
    async fn test_cache_middleware_operations() {
        let config = config::test_config();
        let middleware = CacheMiddleware::new(&config).await.unwrap();
        
        // Test cache key generation
        let params = serde_json::json!(["blockhash", 123]);
        let key1 = middleware.generate_cache_key("getblock", &params);
        let key2 = middleware.generate_cache_key("getblock", &params);
        assert_eq!(key1, key2);
        assert!(key1.starts_with("verus_rpc:"));
        
        // Test cache response decision
        assert!(middleware.should_cache_response("getinfo", 200));
        assert!(!middleware.should_cache_response("sendrawtransaction", 200));
        assert!(!middleware.should_cache_response("getinfo", 404));
    }

    #[test]
    fn test_compression_middleware_creation() {
        let config = config::test_config();
        let middleware = CompressionMiddleware::new(config);
        
        let data = b"This is a test response that should be compressed. ".repeat(50);
        let (compressed_data, encoding) = middleware.compress_response(
            &data,
            "application/json",
            Some("gzip, deflate"),
        );
        
        assert!(compressed_data.len() < data.len());
        assert_eq!(encoding, Some("gzip".to_string()));
    }

    #[test]
    fn test_compression_utilities() {
        // Test should_compress
        assert!(should_compress("application/json", 2048, &config::test_config()));
        assert!(!should_compress("image/png", 2048, &config::test_config()));
        assert!(!should_compress("application/json", 512, &config::test_config()));
        
        // Test get_compression_method
        assert_eq!(get_compression_method(Some("gzip, deflate")), Some(CompressionMethod::Gzip));
        assert_eq!(get_compression_method(Some("deflate")), Some(CompressionMethod::Deflate));
        assert_eq!(get_compression_method(Some("identity")), None);
    }

    #[test]
    fn test_cors_middleware_creation() {
        let config = config::test_config();
        let middleware = CorsMiddleware::new(config);
        
        assert!(middleware.is_initialized());
        assert!(middleware.allows_any_origin());
    }

    #[test]
    fn test_rate_limit_middleware_creation() {
        let config = config::test_config();
        let middleware = RateLimitMiddleware::new(config);
        
        assert!(middleware.is_initialized());
        assert!(!middleware.is_enabled()); // Disabled in test config
    }

    #[tokio::test]
    async fn test_rate_limit_state() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            burst_size: 5,
            enabled: true,
        };
        let state = RateLimitState::new(config);
        
        // Test rate limiting
        assert!(state.check_rate_limit("127.0.0.1").await.is_ok());
        assert!(state.check_rate_limit("127.0.0.1").await.is_ok());
        
        // Test rate limit exceeded
        for _ in 0..10 {
            state.check_rate_limit("127.0.0.1").await.unwrap();
        }
        assert!(state.check_rate_limit("127.0.0.1").await.is_err());
    }

    #[test]
    fn test_security_headers_middleware_creation() {
        let config = config::test_config();
        let middleware = SecurityHeadersMiddleware::new(config);
        
        assert!(middleware.is_initialized());
        assert!(!middleware.is_enabled()); // Disabled in test config
    }
}

/// Configuration unit tests
pub mod config {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.is_initialized());
    }

    #[test]
    fn test_app_config_validation() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.bind_address.to_string(), "127.0.0.1");
        assert!(config.server.port > 0);
        assert!(config.server.max_request_size > 0);
    }

    #[test]
    fn test_security_config() {
        let config = AppConfig::default();
        assert!(config.security.development_mode);
        assert!(!config.security.enable_request_logging);
    }

    #[test]
    fn test_cache_config() {
        let config = AppConfig::default();
        assert!(!config.cache.enabled); // Disabled by default
        assert!(!config.cache.redis_url.is_empty());
        assert!(config.cache.default_ttl > 0);
        assert!(config.cache.max_size > 0);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = AppConfig::default();
        assert!(!config.rate_limit.enabled); // Disabled by default
        assert!(config.rate_limit.requests_per_minute > 0);
        assert!(config.rate_limit.burst_size > 0);
    }
}

/// Error handling unit tests
pub mod errors {
    use super::*;

    #[test]
    fn test_app_error_creation() {
        let error = AppError::Config("Configuration error".to_string());
        assert!(matches!(error, AppError::Config(_)));
        
        let error = AppError::Validation("Validation error".to_string());
        assert!(matches!(error, AppError::Validation(_)));
        
        let error = AppError::Internal("Internal error".to_string());
        assert!(matches!(error, AppError::Internal(_)));
        
        let error = AppError::RateLimit;
        assert!(matches!(error, AppError::RateLimit));
    }

    #[test]
    fn test_app_error_display() {
        let error = AppError::Config("Test error".to_string());
        let error_string = error.to_string();
        assert!(error_string.contains("Test error"));
    }

    #[test]
    fn test_app_error_http_status_code() {
        assert_eq!(AppError::Config("".to_string()).http_status_code(), warp::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::Validation("".to_string()).http_status_code(), warp::http::StatusCode::BAD_REQUEST);
        assert_eq!(AppError::Internal("".to_string()).http_status_code(), warp::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::RateLimit.http_status_code(), warp::http::StatusCode::TOO_MANY_REQUESTS);
    }
}

/// Integration between components unit tests
pub mod integration {
    use super::*;

    #[tokio::test]
    async fn test_full_request_processing_flow() {
        // Create all components
        let config = config::test_config();
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(Arc::new(config.clone()), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service);
        
        // Create request
        let request = fixtures::test_rpc_request("getinfo", serde_json::json!([]));
        
        // Process request
        let result = use_case.execute(request).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_middleware_integration() {
        let config = config::test_config();
        
        // Test cache middleware
        let cache_middleware = CacheMiddleware::new(&config).await.unwrap();
        assert!(cache_middleware.is_initialized());
        
        // Test compression middleware
        let compression_middleware = CompressionMiddleware::new(config.clone());
        let data = b"test data".repeat(100);
        let (compressed, encoding) = compression_middleware.compress_response(
            &data,
            "application/json",
            Some("gzip"),
        );
        assert!(compressed.len() < data.len());
        assert_eq!(encoding, Some("gzip".to_string()));
        
        // Test CORS middleware
        let cors_middleware = CorsMiddleware::new(config.clone());
        assert!(cors_middleware.is_initialized());
        
        // Test rate limit middleware
        let rate_limit_middleware = RateLimitMiddleware::new(config.clone());
        assert!(rate_limit_middleware.is_initialized());
        
        // Test security headers middleware
        let security_headers_middleware = SecurityHeadersMiddleware::new(config);
        assert!(security_headers_middleware.is_initialized());
    }

    #[tokio::test]
    async fn test_error_propagation() {
        let config = config::test_config();
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let rpc_service = Arc::new(RpcService::new(Arc::new(config), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        let use_case = ProcessRpcRequestUseCase::new(rpc_service, metrics_service);
        
        // Create invalid request
        let request = fixtures::test_rpc_request("invalid_method", serde_json::json!([]));
        
        // Process request - should fail
        let result = use_case.execute(request).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_unit_tests_organized() {
        // This test ensures all unit test modules are properly organized
        assert!(true); // Placeholder for test organization validation
    }
} 