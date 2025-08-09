use crate::{
    application::use_cases::health_check::HealthCheckUseCase,
    infrastructure::adapters::ExternalRpcAdapter,
    shared::error::AppResult,
};
use std::sync::Arc;
use warp::{Filter, Reply};
use serde_json::json;

/// Create health check routes
pub fn create_health_routes(
    health_use_case: Arc<HealthCheckUseCase>,
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> impl Filter<Extract = impl Reply> + Clone {
    let health_check = health_check_route(health_use_case.clone(), rpc_adapter.clone());
    let circuit_breaker_status = circuit_breaker_status_route(rpc_adapter.clone());
    let circuit_breaker_reset = circuit_breaker_reset_route(rpc_adapter);
    
    health_check
        .or(circuit_breaker_status)
        .or(circuit_breaker_reset)
}

/// Health check endpoint
fn health_check_route(
    health_use_case: Arc<HealthCheckUseCase>,
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> impl Filter<Extract = impl Reply> + Clone {
    warp::path("health")
        .and(warp::get())
        .and(with_health_use_case(health_use_case))
        .and(with_rpc_adapter(rpc_adapter))
        .and_then(handle_health_check)
}

/// Circuit breaker status endpoint
fn circuit_breaker_status_route(
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> impl Filter<Extract = impl Reply> + Clone {
    warp::path("admin")
        .and(warp::path("circuit-breaker"))
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_rpc_adapter(rpc_adapter))
        .and_then(handle_circuit_breaker_status)
}

/// Circuit breaker reset endpoint
fn circuit_breaker_reset_route(
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> impl Filter<Extract = impl Reply> + Clone {
    warp::path("admin")
        .and(warp::path("circuit-breaker"))
        .and(warp::path("reset"))
        .and(warp::post())
        .and(with_rpc_adapter(rpc_adapter))
        .and_then(handle_circuit_breaker_reset)
}

/// Dependency injection helpers
fn with_health_use_case(
    health_use_case: Arc<HealthCheckUseCase>,
) -> impl Filter<Extract = (Arc<HealthCheckUseCase>,)> + Clone {
    warp::any().map(move || health_use_case.clone())
}

fn with_rpc_adapter(
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> impl Filter<Extract = (Option<Arc<ExternalRpcAdapter>>,)> + Clone {
    warp::any().map(move || rpc_adapter.clone())
}

/// Health check handler
async fn handle_health_check(
    health_use_case: Arc<HealthCheckUseCase>,
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> AppResult<impl Reply> {
    let response = health_use_case.execute(rpc_adapter).await?;
    let status_code = response.http_status_code();
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::from_u16(status_code).unwrap_or(warp::http::StatusCode::OK),
    ))
}

/// Circuit breaker status handler
async fn handle_circuit_breaker_status(
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> AppResult<impl Reply> {
    match rpc_adapter {
        Some(adapter) => {
            let status = adapter.get_circuit_status().await;
            let available = adapter.is_available().await;
            
            let response = json!({
                "circuit_breaker": {
                    "status": format!("{:?}", status),
                    "available": available,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }
            });
            
            Ok(warp::reply::json(&response))
        }
        None => {
            let response = json!({
                "error": "RPC adapter not available",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::SERVICE_UNAVAILABLE,
            ))
        }
    }
}

/// Circuit breaker reset handler
async fn handle_circuit_breaker_reset(
    rpc_adapter: Option<Arc<ExternalRpcAdapter>>,
) -> AppResult<impl Reply> {
    match rpc_adapter {
        Some(adapter) => {
            adapter.reset_circuit_breaker().await;
            
            let response = json!({
                "message": "Circuit breaker reset successfully",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            
            Ok(warp::reply::json(&response))
        }
        None => {
            let response = json!({
                "error": "RPC adapter not available",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::SERVICE_UNAVAILABLE,
            ))
        }
    }
}
