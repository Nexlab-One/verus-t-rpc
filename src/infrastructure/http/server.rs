//! HTTP server implementation for reverse proxy deployment
//! 
//! This module contains the HTTP server implementation optimized for deployment
//! behind a reverse proxy (nginx, Caddy, etc.) that handles SSL, compression, and CORS.

use crate::{
    config::AppConfig,
    shared::error::{AppError, AppResult},
    infrastructure::http::models::{JsonRpcRequest, JsonRpcResponse, RequestContext},
    application::{
        services::{RpcService, MetricsService},
        use_cases::{ProcessRpcRequestUseCase, GetMetricsUseCase, HealthCheckUseCase},
    },
    domain::{security::SecurityValidator, validation::DomainValidator},
    infrastructure::{converters::ModelConverter, adapters::{ExternalRpcAdapter, AuthenticationAdapter}},
    middleware::{
        cache::CacheMiddleware, 
        rate_limit::RateLimitMiddleware, 
        security_headers::{SecurityHeadersMiddleware, add_security_headers_to_response, create_json_response_with_security_headers},
    },
};
use std::sync::Arc;
use tracing::{error, info, instrument, warn, debug};
use warp::{Filter, Reply};

/// HTTP server implementation optimized for reverse proxy deployment
pub struct HttpServer {
    config: AppConfig,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    metrics_use_case: Arc<GetMetricsUseCase>,
    health_use_case: Arc<HealthCheckUseCase>,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
}

impl HttpServer {
    /// Create a new HTTP server instance optimized for reverse proxy deployment
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        // Initialize domain layer
        let security_validator = Arc::new(SecurityValidator::new(Default::default()));
        let _domain_validator = Arc::new(DomainValidator::new());
        
        // Initialize infrastructure layer
        let config_arc = Arc::new(config.clone());
        let _external_rpc_adapter = Arc::new(ExternalRpcAdapter::new(config_arc.clone()));
        let _auth_adapter = Arc::new(AuthenticationAdapter::new(config_arc.clone()));
        
        // Initialize application layer
        let rpc_service = Arc::new(RpcService::new(config_arc.clone(), security_validator));
        let metrics_service = Arc::new(MetricsService::new());
        
        // Initialize use cases
        let rpc_use_case = Arc::new(ProcessRpcRequestUseCase::new(
            rpc_service.clone(),
            metrics_service.clone(),
        ));
        let metrics_use_case = Arc::new(GetMetricsUseCase::new(metrics_service));
        let health_use_case = Arc::new(HealthCheckUseCase);

        // Initialize cache middleware
        let cache_middleware = Arc::new(CacheMiddleware::new(&config).await?);

        // Initialize rate limiting middleware
        let rate_limit_middleware = Arc::new(RateLimitMiddleware::new(config.clone()));

        Ok(Self {
            config,
            rpc_use_case,
            metrics_use_case,
            health_use_case,
            cache_middleware,
            rate_limit_middleware,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Run the HTTP server optimized for reverse proxy deployment
    #[instrument(skip(self))]
    pub async fn run(self) -> AppResult<()> {
        let addr = self.config.server_address();
        info!("Starting HTTP server optimized for reverse proxy deployment on {}", addr);
        info!("SSL/TLS, compression, and CORS should be handled by the reverse proxy");
        
        let addr: std::net::SocketAddr = addr.parse()
            .map_err(|e| AppError::Config(format!("Invalid server address: {}", e)))?;

        let routes = self.create_routes();
        
        info!("Starting HTTP server (reverse proxy mode)");
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// Create the application routes optimized for reverse proxy deployment
    fn create_routes(self) -> impl Filter<Extract = impl Reply> + Clone {
        let config = self.config.clone();
        let rpc_use_case = self.rpc_use_case.clone();
        let metrics_use_case = self.metrics_use_case.clone();
        let health_use_case = self.health_use_case.clone();

        // Main RPC endpoint
        let rpc_route = warp::path::end()
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(with_rpc_use_case(rpc_use_case))
            .and(with_config(config.clone()))
            .and(with_cache_middleware(self.cache_middleware.clone()))
            .and(with_rate_limit_middleware(self.rate_limit_middleware.clone()))
            .and_then(handle_rpc_request);

        // Health check endpoint
        let health_route = warp::path("health")
            .and(warp::get())
            .and(with_health_use_case(health_use_case))
            .and(with_config(config.clone()))
            .and_then(handle_health_request);

        // Metrics endpoint
        let metrics_route = warp::path("metrics")
            .and(warp::get())
            .and(with_metrics_use_case(metrics_use_case))
            .and(with_config(config.clone()))
            .and_then(handle_metrics_request);

        // Prometheus metrics endpoint
        let prometheus_route = warp::path("prometheus")
            .and(warp::get())
            .and(with_prometheus_adapter())
            .and(with_config(config.clone()))
            .and_then(handle_prometheus_request);

        // Mining pool share validation endpoint
        let mining_pool_route = warp::path("pool")
            .and(warp::path("share"))
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::header::<String>("x-forwarded-for"))
            .and(with_mining_pool_client())
            .and(with_config(config.clone()))
            .and(with_cache_middleware(self.cache_middleware.clone()))
            .and(with_rate_limit_middleware(self.rate_limit_middleware.clone()))
            .and_then(handle_mining_pool_request);

        // Mining pool metrics endpoint
        let pool_metrics_route = warp::path("pool")
            .and(warp::path("metrics"))
            .and(warp::get())
            .and(with_mining_pool_client())
            .and(with_config(config.clone()))
            .and_then(handle_pool_metrics_request);

        // Combine all routes
        rpc_route.or(health_route).or(metrics_route).or(prometheus_route).or(mining_pool_route).or(pool_metrics_route)
    }
}

/// Handle RPC requests optimized for reverse proxy deployment
#[instrument(skip(rpc_use_case, config, cache_middleware, rate_limit_middleware))]
async fn handle_rpc_request(
    request: JsonRpcRequest,
    client_ip: String,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
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
            "Processing RPC request"
        );
    }

    // Validate request
    if let Err(e) = request.validate_request() {
        error!(
            request_id = %context.request_id,
            error = %e,
            "Request validation failed"
        );
        let error_response = JsonRpcResponse::error(
            crate::infrastructure::http::models::JsonRpcError::invalid_request(),
            request.id,
        );
        
        // Apply security headers
        let security_middleware = SecurityHeadersMiddleware::new(config.clone());
        let response = create_json_response_with_security_headers(
            &error_response,
            &security_middleware,
        );
        
        return Ok(warp::reply::with_status(
            response,
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check rate limit
    if rate_limit_middleware.is_enabled() {
        let client_ip = validated_client_ip.clone();
        let client_limiter = rate_limit_middleware.create_client_limiter(&client_ip);
        if let Err(e) = client_limiter.check_rate_limit(&client_ip).await {
            error!(
                request_id = %context.request_id,
                client_ip = %validated_client_ip,
                error = %e,
                "Rate limit exceeded"
            );
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error("Rate limit exceeded"),
                request.id,
            );
            
            // Apply security headers
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::TOO_MANY_REQUESTS,
            ));
        }
    }

    // Check cache for read-only methods
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
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::OK,
            ));
        }
    }

    // Convert to domain model
    let domain_request = match ModelConverter::to_domain_request(&request, &context) {
        Ok(req) => req,
        Err(e) => {
            error!(
                request_id = %context.request_id,
                error = %e,
                "Failed to convert request to domain model"
            );
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error(&e.to_string()),
                request.id,
            );
            
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Process request using use case
    let request_id = request.id.clone();
    let context_request_id = context.request_id.clone();
    match rpc_use_case.execute(domain_request).await {
        Ok(domain_response) => {
            info!(
                request_id = %context_request_id,
                "Request processed successfully"
            );
            
            // Convert domain response to infrastructure response
            let infra_response = ModelConverter::to_infrastructure_response(&domain_response);
            
            // Cache the response if it's a cacheable method
            if cache_middleware.should_cache_response(&request.method, 200) {
                let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
                let cache_key = cache_middleware.generate_cache_key(&request.method, params);
                
                // Serialize response for caching
                if let Ok(response_data) = serde_json::to_vec(&infra_response) {
                    let cache_entry = cache_middleware.create_cache_entry(
                        cache_key,
                        response_data,
                        "application/json".to_string(),
                        config.cache.default_ttl,
                    );
                    
                    // Cache the response (fire and forget)
                    if let Err(e) = cache_middleware.cache_response(cache_entry).await {
                        warn!(
                            request_id = %context_request_id,
                            error = %e,
                            "Failed to cache response"
                        );
                    } else {
                        debug!(
                            request_id = %context_request_id,
                            method = %request.method,
                            "Response cached successfully"
                        );
                    }
                }
            }
            
            // Apply security headers
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &infra_response,
                &security_middleware,
            );
            
            Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            error!(
                request_id = %context_request_id,
                error = %e,
                "Request processing failed"
            );
            
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error(&e.to_string()),
                request_id,
            );
            
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            Ok(warp::reply::with_status(
                response,
                e.http_status_code(),
            ))
        }
    }
}

/// Handle health check requests
async fn handle_health_request(
    health_use_case: Arc<HealthCheckUseCase>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let health_data = health_use_case.execute();
    
    // Apply security headers only
    let response = create_json_response_with_security_headers(
        &health_data,
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

/// Handle metrics requests
async fn handle_metrics_request(
    metrics_use_case: Arc<GetMetricsUseCase>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let metrics_data = metrics_use_case.execute();
    
    // Apply security headers only
    let response = create_json_response_with_security_headers(
        &metrics_data,
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

/// Handle Prometheus metrics requests
async fn handle_prometheus_request(
    monitoring_adapter: Arc<crate::infrastructure::adapters::MonitoringAdapter>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let metrics = monitoring_adapter.get_prometheus_metrics();
    
    // Apply security headers only
    let response = add_security_headers_to_response(
        warp::reply::with_header(
            warp::reply::with_status(metrics, warp::http::StatusCode::OK),
            "Content-Type",
            "text/plain; version=0.0.4; charset=utf-8"
        ),
        &SecurityHeadersMiddleware::new(config.clone()),
    );
    
    Ok(response)
}

/// Handle mining pool share validation requests
async fn handle_mining_pool_request(
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

    // Validate request
    if let Err(e) = request.validate_request() {
        error!(
            request_id = %context.request_id,
            error = %e,
            "Request validation failed"
        );
        let error_response = JsonRpcResponse::error(
            crate::infrastructure::http::models::JsonRpcError::invalid_request(),
            request.id,
        );
        
        // Apply security headers
        let security_middleware = SecurityHeadersMiddleware::new(config.clone());
        let response = create_json_response_with_security_headers(
            &error_response,
            &security_middleware,
        );
        
        return Ok(warp::reply::with_status(
            response,
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check rate limit
    if rate_limit_middleware.is_enabled() {
        let client_ip = validated_client_ip.clone();
        let client_limiter = rate_limit_middleware.create_client_limiter(&client_ip);
        if let Err(e) = client_limiter.check_rate_limit(&client_ip).await {
            error!(
                request_id = %context.request_id,
                client_ip = %validated_client_ip,
                error = %e,
                "Rate limit exceeded"
            );
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error("Rate limit exceeded"),
                request.id,
            );
            
            // Apply security headers
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::TOO_MANY_REQUESTS,
            ));
        }
    }

    // Check cache for read-only methods
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
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::OK,
            ));
        }
    }

    // Convert to domain model
    let domain_request = match ModelConverter::to_domain_request(&request, &context) {
        Ok(req) => req,
        Err(e) => {
            error!(
                request_id = %context.request_id,
                error = %e,
                "Failed to convert request to domain model"
            );
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error(&e.to_string()),
                request.id,
            );
            
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Process request using use case
    let request_id = request.id.clone();
    let context_request_id = context.request_id.clone();
    
    // Parse pool share from domain request parameters
    let pool_share = match parse_pool_share_from_request(&domain_request) {
        Ok(share) => share,
        Err(e) => {
            error!(
                request_id = %context_request_id,
                error = %e,
                "Failed to parse pool share from request parameters"
            );
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::invalid_params(&domain_request.method, &e.to_string()),
                request_id,
            );
            
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            return Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };
     
     match mining_pool_client.validate_share(&pool_share).await {
                 Ok(domain_response) => {
             info!(
                 request_id = %context_request_id,
                 "Mining pool share validation request processed successfully"
             );
             
             // Create a JSON-RPC response from the pool validation response
             let result = serde_json::to_value(domain_response)
                 .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize response"}));
             
             let infra_response = JsonRpcResponse::success(result, request_id);
            
            // Cache the response if it's a cacheable method
            if cache_middleware.should_cache_response(&request.method, 200) {
                let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
                let cache_key = cache_middleware.generate_cache_key(&request.method, params);
                
                // Serialize response for caching
                if let Ok(response_data) = serde_json::to_vec(&infra_response) {
                    let cache_entry = cache_middleware.create_cache_entry(
                        cache_key,
                        response_data,
                        "application/json".to_string(),
                        config.cache.default_ttl,
                    );
                    
                    // Cache the response (fire and forget)
                    if let Err(e) = cache_middleware.cache_response(cache_entry).await {
                        warn!(
                            request_id = %context_request_id,
                            error = %e,
                            "Failed to cache response"
                        );
                    } else {
                        debug!(
                            request_id = %context_request_id,
                            method = %request.method,
                            "Response cached successfully"
                        );
                    }
                }
            }
            
            // Apply security headers
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &infra_response,
                &security_middleware,
            );
            
            Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            error!(
                request_id = %context_request_id,
                error = %e,
                "Mining pool share validation request processing failed"
            );
            
            let error_response = JsonRpcResponse::error(
                crate::infrastructure::http::models::JsonRpcError::internal_error(&e.to_string()),
                request_id,
            );
            
            let security_middleware = SecurityHeadersMiddleware::new(config.clone());
            let response = create_json_response_with_security_headers(
                &error_response,
                &security_middleware,
            );
            
            Ok(warp::reply::with_status(
                response,
                e.http_status_code(),
            ))
        }
    }
}

/// Handle mining pool metrics requests
async fn handle_pool_metrics_request(
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

/// Helper function to inject RPC use case into route
fn with_rpc_use_case(
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
) -> impl Filter<Extract = (Arc<ProcessRpcRequestUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rpc_use_case.clone())
}

/// Helper function to inject health use case into route
fn with_health_use_case(
    health_use_case: Arc<HealthCheckUseCase>,
) -> impl Filter<Extract = (Arc<HealthCheckUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || health_use_case.clone())
}

/// Helper function to inject metrics use case into route
fn with_metrics_use_case(
    metrics_use_case: Arc<GetMetricsUseCase>,
) -> impl Filter<Extract = (Arc<GetMetricsUseCase>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics_use_case.clone())
}

/// Helper function to inject mining pool client into route
fn with_mining_pool_client(
) -> impl Filter<Extract = (Arc<crate::infrastructure::adapters::MiningPoolClient>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || {
        // Create a default config for the mining pool client
        let mut config = crate::config::AppConfig::default();
        config.security.mining_pool = Some(crate::config::app_config::MiningPoolConfig {
            pool_url: "https://test-pool.com".to_string(),
            api_key: "test-key".to_string(),
            public_key: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            requests_per_minute: 100,
            enabled: true,
        });
        Arc::new(crate::infrastructure::adapters::MiningPoolClient::new(Arc::new(config)))
    })
}

/// Extract and validate client IP from various sources
fn extract_and_validate_client_ip(raw_ip: &str, config: &AppConfig) -> String {
    // If the IP is empty or invalid, return a default
    if raw_ip.is_empty() || raw_ip == "unknown" {
        return "127.0.0.1".to_string();
    }
    
    // Parse the IP to validate it
    if let Ok(ip) = raw_ip.parse::<std::net::IpAddr>() {
        // Check if it's a private/local IP and if we should trust it
        if config.security.trusted_proxy_headers.contains(&"X-Forwarded-For".to_string()) {
            // If we trust proxy headers, return the IP as-is
            return ip.to_string();
        } else {
            // If we don't trust proxy headers, only accept local IPs
            if ip.is_loopback() {
                return ip.to_string();
            } else {
                return "127.0.0.1".to_string();
            }
        }
    }
    
    // If parsing failed, return default
    "127.0.0.1".to_string()
}

/// Helper function to inject Prometheus adapter into route
fn with_prometheus_adapter(
) -> impl Filter<Extract = (Arc<crate::infrastructure::adapters::MonitoringAdapter>,), Error = std::convert::Infallible> + Clone {
    let monitoring_adapter = Arc::new(crate::infrastructure::adapters::MonitoringAdapter::new());
    warp::any().map(move || monitoring_adapter.clone())
}

/// Helper function to inject configuration into route
fn with_config(
    config: AppConfig,
) -> impl Filter<Extract = (AppConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

/// Helper function to inject cache middleware into route
fn with_cache_middleware(
    cache_middleware: Arc<CacheMiddleware>,
) -> impl Filter<Extract = (Arc<CacheMiddleware>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cache_middleware.clone())
}

/// Helper function to inject rate limiting middleware into route
fn with_rate_limit_middleware(
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> impl Filter<Extract = (Arc<RateLimitMiddleware>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rate_limit_middleware.clone())
}

/// Parse pool share from domain request parameters
fn parse_pool_share_from_request(domain_request: &crate::domain::rpc::RpcRequest) -> AppResult<crate::infrastructure::adapters::PoolShare> {
    use crate::shared::error::AppError;
    use serde_json::Value;
    
    // Extract parameters from the domain request
    let params = domain_request.parameters.as_ref()
        .ok_or_else(|| AppError::Validation("Missing request parameters".to_string()))?;
    
    // Parse the parameters as a JSON object
    let params_obj = params.as_object()
        .ok_or_else(|| AppError::Validation("Parameters must be a JSON object".to_string()))?;
    
    // Extract required fields with validation
    let challenge_id = params_obj.get("challenge_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid challenge_id".to_string()))?
        .to_string();
    
    let miner_address = params_obj.get("miner_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid miner_address".to_string()))?
        .to_string();
    
    let nonce = params_obj.get("nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid nonce".to_string()))?
        .to_string();
    
    let solution = params_obj.get("solution")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing or invalid solution".to_string()))?
        .to_string();
    
    let difficulty = params_obj.get("difficulty")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::Validation("Missing or invalid difficulty".to_string()))?;
    
    // Parse timestamp (accept both ISO string and Unix timestamp)
    let timestamp = if let Some(timestamp_value) = params_obj.get("timestamp") {
        match timestamp_value {
            Value::String(timestamp_str) => {
                // Try to parse as ISO 8601 string
                chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .or_else(|_| {
                        // Try to parse as Unix timestamp
                        timestamp_str.parse::<i64>()
                            .ok()
                            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            .ok_or_else(|| AppError::Validation("Invalid timestamp format".to_string()))
                    })?
            }
            Value::Number(timestamp_num) => {
                // Parse as Unix timestamp
                timestamp_num.as_i64()
                    .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                    .ok_or_else(|| AppError::Validation("Invalid timestamp value".to_string()))?
            }
            _ => return Err(AppError::Validation("Invalid timestamp format".to_string())),
        }
    } else {
        // Use current timestamp if not provided
        chrono::Utc::now()
    };
    
    // Parse optional pool signature
    let pool_signature = params_obj.get("pool_signature")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Validate field lengths and values
    if challenge_id.is_empty() {
        return Err(AppError::Validation("challenge_id cannot be empty".to_string()));
    }
    if miner_address.is_empty() {
        return Err(AppError::Validation("miner_address cannot be empty".to_string()));
    }
    if nonce.is_empty() {
        return Err(AppError::Validation("nonce cannot be empty".to_string()));
    }
    if solution.is_empty() {
        return Err(AppError::Validation("solution cannot be empty".to_string()));
    }
    if difficulty <= 0.0 {
        return Err(AppError::Validation("difficulty must be positive".to_string()));
    }
    
    Ok(crate::infrastructure::adapters::PoolShare {
        challenge_id,
        miner_address,
        nonce,
        solution,
        difficulty,
        timestamp,
        pool_signature,
    })
}

#[cfg(test)]
/// Create test routes for integration testing
pub async fn create_test_routes() -> Result<impl Filter<Extract = impl Reply> + Clone, Box<dyn std::error::Error + Send + Sync>> {
    // For testing, we'll create a simple configuration
    let mut config = AppConfig::default();
    config.server.port = 0; // Use random port
    config.server.bind_address = "127.0.0.1".parse().unwrap();
    config.security.development_mode = true;
    config.cache.enabled = false;
    config.rate_limit.enabled = false;
    
    let server = HttpServer::new(config).await?;
    Ok(server.create_routes())
} 