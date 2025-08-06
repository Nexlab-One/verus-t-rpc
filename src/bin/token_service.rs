//! Token Issuance Service
//! 
//! A separate service for issuing and validating JWT tokens.
//! This service can be deployed independently from the main RPC server.

use verus_rpc_server::{
    config::AppConfig,
    infrastructure::adapters::{
        TokenIssuerAdapter, TokenIssuanceRequest,
        TokenValidationRequest
    },
    shared::error::{AppResult, AppError},
};
use std::sync::Arc;
use tracing::{info, error};
use warp::{Filter, Reply};
use serde::{Deserialize, Serialize};

/// Token service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenServiceConfig {
    /// Service port
    pub port: u16,
    
    /// Bind address
    pub bind_address: String,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,
    
    /// Rate limiting
    pub rate_limit_requests_per_minute: u32,
    
    /// Enable request logging
    pub enable_request_logging: bool,
}

impl Default for TokenServiceConfig {
    fn default() -> Self {
        Self {
            port: 8081,
            bind_address: "127.0.0.1".to_string(),
            enable_cors: true,
            allowed_origins: vec!["*".to_string()],
            rate_limit_requests_per_minute: 100,
            enable_request_logging: true,
        }
    }
}

/// Token service
pub struct TokenService {
    config: TokenServiceConfig,
    app_config: Arc<AppConfig>,
    token_issuer: Arc<TokenIssuerAdapter>,
}

impl TokenService {
    /// Create a new token service
    pub fn new(config: TokenServiceConfig, app_config: AppConfig) -> Self {
        let app_config = Arc::new(app_config);
        let token_issuer = Arc::new(TokenIssuerAdapter::new(app_config.clone()));
        
        Self {
            config,
            app_config,
            token_issuer,
        }
    }

    /// Run the token service
    pub async fn run(self) -> AppResult<()> {
        let addr = format!("{}:{}", self.config.bind_address, self.config.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| AppError::Config(format!("Invalid address: {}", e)))?;

        info!("Starting Token Issuance Service on {}", addr);
        
        let routes = self.create_routes();
        
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// Create the service routes
    fn create_routes(self) -> impl Filter<Extract = impl Reply> + Clone {
        let app_config = self.app_config.clone();
        let token_issuer = self.token_issuer.clone();

        // Health check endpoint
        let health_route = warp::path("health")
            .and(warp::get())
            .map(|| {
                let response = serde_json::json!({
                    "status": "healthy",
                    "service": "token-issuance",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                warp::reply::json(&response)
            });

        // Issue token endpoint
        let issue_token_route = warp::path("issue")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_token_issuer(token_issuer.clone()))
            .and(with_app_config(app_config.clone()))
            .and_then(handle_issue_token);

        // Validate token endpoint
        let validate_token_route = warp::path("validate")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_token_issuer(token_issuer))
            .and(with_app_config(app_config))
            .and_then(handle_validate_token);

        // Combine routes
        let routes = health_route
            .or(issue_token_route)
            .or(validate_token_route);

        // Apply CORS
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["Content-Type", "Authorization"]);
        
        routes.with(cors)
    }
}

/// Handle token issuance request
async fn handle_issue_token(
    request: TokenIssuanceRequest,
    token_issuer: Arc<TokenIssuerAdapter>,
    _app_config: Arc<AppConfig>,
) -> Result<impl Reply, warp::reject::Rejection> {
    info!("Processing token issuance request for user: {}", request.user_id);

    match token_issuer.issue_token(request).await {
        Ok(response) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            error!("Token issuance failed: {}", e);
            let error_response = serde_json::json!({
                "error": "token_issuance_failed",
                "message": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    }
}

/// Handle token validation request
async fn handle_validate_token(
    request: TokenValidationRequest,
    token_issuer: Arc<TokenIssuerAdapter>,
    _app_config: Arc<AppConfig>,
) -> Result<impl Reply, warp::reject::Rejection> {
    info!("Processing token validation request");

    match token_issuer.validate_token(request).await {
        Ok(response) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            error!("Token validation failed: {}", e);
            let error_response = serde_json::json!({
                "error": "token_validation_failed",
                "message": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    }
}

/// Dependency injection for token issuer
fn with_token_issuer(
    token_issuer: Arc<TokenIssuerAdapter>,
) -> impl Filter<Extract = (Arc<TokenIssuerAdapter>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || token_issuer.clone())
}

/// Dependency injection for app config
fn with_app_config(
    app_config: Arc<AppConfig>,
) -> impl Filter<Extract = (Arc<AppConfig>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app_config.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Token Issuance Service");

    // Load configuration
    let app_config = AppConfig::default();
    let token_service_config = TokenServiceConfig::default();

    // Create and run the service
    let service = TokenService::new(token_service_config, app_config);
    
    if let Err(e) = service.run().await {
        error!("Token service failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
