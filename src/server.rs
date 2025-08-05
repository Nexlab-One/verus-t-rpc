use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    handlers::RpcHandler,
    models::{JsonRpcRequest, JsonRpcResponse, RequestContext},
};
use std::sync::Arc;
use tracing::{error, info, instrument};
use warp::{Filter, Reply};

/// Main server implementation
pub struct VerusRpcServer {
    config: AppConfig,
    rpc_handler: Arc<RpcHandler>,
}

impl VerusRpcServer {
    /// Create a new server instance
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let rpc_handler = Arc::new(RpcHandler::new(&config).await?);
        
        Ok(Self {
            config,
            rpc_handler,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Run the server
    #[instrument(skip(self))]
    pub async fn run(self) -> AppResult<()> {
        let addr = self.config.server_address();
        info!("Starting server on {}", addr);
        
        let addr: std::net::SocketAddr = addr.parse()
            .map_err(|e| AppError::Config(format!("Invalid server address: {}", e)))?;

        let routes = self.create_routes();
        
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// Create the application routes
    fn create_routes(self) -> impl Filter<Extract = impl Reply> + Clone {
        let config = self.config.clone();
        let rpc_handler = self.rpc_handler.clone();

        // Main RPC endpoint
        let rpc_route = warp::path::end()
            .and(warp::post())
            .and(warp::body::content_length_limit(config.server.max_request_size as u64))
            .and(warp::body::json())
            .and(with_rpc_handler(rpc_handler))
            .and(with_config(config.clone()))
            .and_then(handle_rpc_request);

        // Health check endpoint
        let health_route = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

        // Metrics endpoint
        let metrics_route = warp::path("metrics")
            .and(warp::get())
            .and(with_config(config))
            .and_then(handle_metrics_request);

        rpc_route.or(health_route).or(metrics_route)
    }
}

/// Handle RPC requests
#[instrument(skip(rpc_handler, config))]
async fn handle_rpc_request(
    request: JsonRpcRequest,
    rpc_handler: Arc<RpcHandler>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Create request context
    let context = RequestContext::new(
        "127.0.0.1".to_string(), // TODO: Extract from request
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
        return Ok(warp::reply::with_status(
            warp::reply::json(&JsonRpcResponse::error(
                crate::models::JsonRpcError::invalid_request(),
                request.id,
            )),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Process request
    let request_id = request.id.clone();
    let context_request_id = context.request_id.clone();
    match rpc_handler.handle_request(request, context).await {
        Ok(response) => {
            info!(
                request_id = %context_request_id,
                "Request processed successfully"
            );
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
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
                crate::models::JsonRpcError::internal_error(&e.to_string()),
                request_id,
            );
            
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                e.http_status_code(),
            ))
        }
    }
}

/// Handle metrics requests
async fn handle_metrics_request(
    _config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    // TODO: Implement metrics collection
    let metrics = serde_json::json!({
        "total_requests": 0,
        "successful_requests": 0,
        "failed_requests": 0,
        "rate_limited_requests": 0,
        "avg_response_time_ms": 0.0,
        "active_connections": 0,
        "uptime_seconds": 0,
    });

    Ok(warp::reply::json(&metrics))
}

/// Helper function to inject RPC handler into route
fn with_rpc_handler(
    rpc_handler: Arc<RpcHandler>,
) -> impl Filter<Extract = (Arc<RpcHandler>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rpc_handler.clone())
}

/// Helper function to inject configuration into route
fn with_config(
    config: AppConfig,
) -> impl Filter<Extract = (AppConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
} 