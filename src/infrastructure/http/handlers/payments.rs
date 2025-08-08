//! Payments HTTP handlers

use std::sync::Arc;

use warp::Reply;

use crate::application::services::payments_service::{PaymentQuoteRequest, PaymentSubmitRequest, PaymentsService};
use crate::config::AppConfig;
use crate::infrastructure::http::models::RequestContext;
use crate::domain::rpc::ClientInfo;
use crate::middleware::security_headers::{create_json_response_with_security_headers, SecurityHeadersMiddleware};
use crate::middleware::rate_limit::{RateLimitMiddleware};

pub async fn handle_payment_quote(
    body: PaymentQuoteRequest,
    client_ip: String,
    service: Arc<PaymentsService>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Apply per-IP rate limit using global settings
    let limiter = RateLimitMiddleware::new(config.clone()).create_client_limiter(&client_ip);
    if let Err(_) = limiter.check_rate_limit(&client_ip).await {
        let resp = create_json_response_with_security_headers(&serde_json::json!({"error":"Rate limit"}), &SecurityHeadersMiddleware::new(config.clone()));
        return Ok(warp::reply::with_status(resp, warp::http::StatusCode::TOO_MANY_REQUESTS));
    }
    let context = RequestContext::new(client_ip.clone(), "payments.quote".to_string(), None);
    let client_info = ClientInfo {
        ip_address: context.client_ip.clone(),
        user_agent: context.user_agent.clone(),
        auth_token: None,
        timestamp: context.timestamp,
    };
    let result = service.create_quote(body, &client_info).await;

    let response = match result {
        Ok(resp) => warp::reply::with_status(
            create_json_response_with_security_headers(&resp, &SecurityHeadersMiddleware::new(config.clone())),
            warp::http::StatusCode::OK,
        ),
        Err(e) => warp::reply::with_status(
            create_json_response_with_security_headers(&serde_json::json!({ "error": e.to_string() }), &SecurityHeadersMiddleware::new(config.clone())),
            e.http_status_code(),
        ),
    };
    Ok(response)
}

pub async fn handle_payment_submit(
    body: PaymentSubmitRequest,
    client_ip: String,
    service: Arc<PaymentsService>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let limiter = RateLimitMiddleware::new(config.clone()).create_client_limiter(&client_ip);
    if let Err(_) = limiter.check_rate_limit(&client_ip).await {
        let resp = create_json_response_with_security_headers(&serde_json::json!({"error":"Rate limit"}), &SecurityHeadersMiddleware::new(config.clone()));
        return Ok(warp::reply::with_status(resp, warp::http::StatusCode::TOO_MANY_REQUESTS));
    }
    let context = RequestContext::new(client_ip.clone(), "payments.submit".to_string(), None);
    let client_info = ClientInfo {
        ip_address: context.client_ip.clone(),
        user_agent: context.user_agent.clone(),
        auth_token: None,
        timestamp: context.timestamp,
    };
    let result = service.submit_raw_transaction(body, &client_info).await;
    let response = match result {
        Ok(resp) => warp::reply::with_status(
            create_json_response_with_security_headers(&resp, &SecurityHeadersMiddleware::new(config.clone())),
            warp::http::StatusCode::OK,
        ),
        Err(e) => warp::reply::with_status(
            create_json_response_with_security_headers(&serde_json::json!({ "error": e.to_string() }), &SecurityHeadersMiddleware::new(config.clone())),
            e.http_status_code(),
        ),
    };
    Ok(response)
}

pub async fn handle_payment_status(
    payment_id: String,
    client_ip: String,
    service: Arc<PaymentsService>,
    config: AppConfig,
) -> Result<impl Reply, warp::reject::Rejection> {
    let limiter = RateLimitMiddleware::new(config.clone()).create_client_limiter(&client_ip);
    if let Err(_) = limiter.check_rate_limit(&client_ip).await {
        let resp = create_json_response_with_security_headers(&serde_json::json!({"error":"Rate limit"}), &SecurityHeadersMiddleware::new(config.clone()));
        return Ok(warp::reply::with_status(resp, warp::http::StatusCode::TOO_MANY_REQUESTS));
    }
    let context = RequestContext::new(client_ip.clone(), "payments.status".to_string(), None);
    let client_info = ClientInfo {
        ip_address: context.client_ip.clone(),
        user_agent: context.user_agent.clone(),
        auth_token: None,
        timestamp: context.timestamp,
    };
    let result = service.check_status(&payment_id, &client_info).await;
    let response = match result {
        Ok(resp) => warp::reply::with_status(
            create_json_response_with_security_headers(&resp, &SecurityHeadersMiddleware::new(config.clone())),
            warp::http::StatusCode::OK,
        ),
        Err(e) => warp::reply::with_status(
            create_json_response_with_security_headers(&serde_json::json!({ "error": e.to_string() }), &SecurityHeadersMiddleware::new(config.clone())),
            e.http_status_code(),
        ),
    };
    Ok(response)
}


