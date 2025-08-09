use crate::{
    domain::rpc::*,
    application::services::rpc_service::RpcService,
    application::services::metrics_service::MetricsService,
    shared::error::AppResult,
};
use std::sync::Arc;
use serde_json::json;
use tracing::{info, warn};

/// RPC request processing use case
pub struct ProcessRpcRequestUseCase {
    rpc_service: Arc<RpcService>,
    metrics_service: Arc<MetricsService>,
}

impl ProcessRpcRequestUseCase {
    /// Create a new RPC request processing use case
    pub fn new(rpc_service: Arc<RpcService>, metrics_service: Arc<MetricsService>) -> Self {
        Self {
            rpc_service,
            metrics_service,
        }
    }

    /// Execute RPC request with fallback strategy
    pub async fn execute(&self, request: RpcRequest) -> AppResult<RpcResponse> {
        info!(
            method = %request.method,
            client_ip = %request.client_info.ip_address,
            "Processing RPC request"
        );

        // Record metrics
        self.metrics_service.record_request(&request.method).await;

        // Try to process the request
        match self.rpc_service.process_request(&request).await {
            Ok(response) => {
                self.metrics_service.record_success(&request.method).await;
                Ok(response)
            }
            Err(error) => {
                self.metrics_service.record_error(&request.method).await;
                
                // Check if this is a daemon connectivity error
                if self.is_daemon_connectivity_error(&error) {
                    warn!("Daemon connectivity error, providing fallback response");
                    self.provide_fallback_response(&request).await
                } else {
                    Err(error)
                }
            }
        }
    }

    /// Check if the error is related to daemon connectivity
    fn is_daemon_connectivity_error(&self, error: &crate::shared::error::AppError) -> bool {
        match error {
            crate::shared::error::AppError::Rpc(msg) => {
                msg.contains("Service temporarily unavailable") ||
                msg.contains("circuit breaker open") ||
                msg.contains("Failed to connect") ||
                msg.contains("Connection refused") ||
                msg.contains("timeout")
            }
            _ => false,
        }
    }

    /// Provide fallback response when daemon is unavailable
    async fn provide_fallback_response(&self, request: &RpcRequest) -> AppResult<RpcResponse> {
        match request.method.as_str() {
            "getinfo" => {
                // Provide cached or default blockchain info
                let fallback_info = json!({
                    "version": "0.1.0",
                    "protocolversion": 170002,
                    "walletversion": 60000,
                    "balance": 0.0,
                    "blocks": 0,
                    "timeoffset": 0,
                    "connections": 0,
                    "proxy": "",
                    "difficulty": 0.0,
                    "testnet": true,
                    "keypoololdest": 0,
                    "keypoolsize": 0,
                    "paytxfee": 0.0,
                    "relayfee": 0.0,
                    "errors": "Daemon temporarily unavailable - using fallback data"
                });
                
                Ok(RpcResponse::success(fallback_info, request.id.clone()))
            }
            "getblockchaininfo" => {
                let fallback_info = json!({
                    "chain": "test",
                    "blocks": 0,
                    "headers": 0,
                    "bestblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "difficulty": 0.0,
                    "mediantime": 0,
                    "verificationprogress": 0.0,
                    "initialblockdownload": true,
                    "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
                    "size_on_disk": 0,
                    "pruned": false,
                    "pruneheight": 0,
                    "automatic_pruning": false,
                    "prune_target_size": 0,
                    "warnings": "Daemon temporarily unavailable - using fallback data"
                });
                
                Ok(RpcResponse::success(fallback_info, request.id.clone()))
            }
            "getnetworkinfo" => {
                let fallback_info = json!({
                    "version": 170002,
                    "subversion": "/Verus:0.1.0/",
                    "protocolversion": 170002,
                    "localservices": "0000000000000000",
                    "localservicesnames": [],
                    "localrelay": true,
                    "timeoffset": 0,
                    "networkactive": false,
                    "connections": 0,
                    "networks": [],
                    "relayfee": 0.0,
                    "incrementalfee": 0.0,
                    "localaddresses": [],
                    "warnings": "Daemon temporarily unavailable - using fallback data"
                });
                
                Ok(RpcResponse::success(fallback_info, request.id.clone()))
            }
            "getwalletinfo" => {
                let fallback_info = json!({
                    "walletname": "default",
                    "walletversion": 60000,
                    "balance": 0.0,
                    "unconfirmed_balance": 0.0,
                    "immature_balance": 0.0,
                    "txcount": 0,
                    "keypoololdest": 0,
                    "keypoolsize": 0,
                    "keypoolsize_hd_internal": 0,
                    "unlocked_until": 0,
                    "paytxfee": 0.0,
                    "hdseedid": "0000000000000000000000000000000000000000000000000000000000000000",
                    "private_keys_enabled": true,
                    "avoid_reuse": false,
                    "scanning": false,
                    "descriptors": false,
                    "warnings": "Daemon temporarily unavailable - using fallback data"
                });
                
                Ok(RpcResponse::success(fallback_info, request.id.clone()))
            }
            _ => {
                // For other methods, return a generic error with helpful message
                let error_response = json!({
                    "error": {
                        "code": -32000,
                        "message": "Daemon temporarily unavailable",
                        "details": "The Verus daemon is currently unavailable. Please try again later or contact support if the issue persists."
                    }
                });
                
                Ok(RpcResponse::error(
                    "Daemon temporarily unavailable".to_string(),
                    -32000,
                    request.id.clone(),
                    Some(error_response)
                ))
            }
        }
    }
}
