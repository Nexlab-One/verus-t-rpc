//! External RPC adapter for communicating with Verus daemon
//! 
//! This adapter handles HTTP communication with the external Verus daemon,
//! providing forward compatibility for future Rust daemon integration.

use crate::{
    domain::rpc::*,
    shared::error::AppResult,
    config::AppConfig,
};
use std::sync::Arc;
use tracing::info;

/// Adapter for external RPC services
pub struct ExternalRpcAdapter {
    _config: Arc<AppConfig>,
}

impl ExternalRpcAdapter {
    /// Create a new external RPC adapter
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { _config: config }
    }

    /// Send request to external RPC service
    pub async fn send_request(&self, request: &RpcRequest) -> AppResult<RpcResponse> {
        use reqwest::Client;
        use serde_json::json;
        use std::time::Duration;
        
        info!(
            method = %request.method,
            client_ip = %request.client_info.ip_address,
            "Sending request to external RPC service"
        );

        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(self._config.verus.timeout_seconds))
            .build()
            .map_err(|e| crate::shared::error::AppError::Config(format!("Failed to create HTTP client: {}", e)))?;

        // Create JSON-RPC request payload
        let payload = json!({
            "jsonrpc": "2.0",
            "method": request.method,
            "params": request.parameters,
            "id": request.id
        });

        // Send request with retries
        let mut last_error = None;
        for attempt in 0..=self._config.verus.max_retries {
            match client
                .post(&self._config.verus.rpc_url)
                .header("Content-Type", "application/json")
                .basic_auth(&self._config.verus.rpc_user, Some(&self._config.verus.rpc_password))
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json_response) => {
                                if let Some(result) = json_response.get("result") {
                                    return Ok(RpcResponse::success(result.clone(), request.id.clone()));
                                } else if let Some(error) = json_response.get("error") {
                                    return Err(crate::shared::error::AppError::Rpc(format!("RPC error: {}", error)));
                                } else {
                                    return Err(crate::shared::error::AppError::Rpc("Invalid RPC response".to_string()));
                                }
                            }
                            Err(e) => {
                                last_error = Some(format!("Failed to parse response: {}", e));
                            }
                        }
                    } else {
                        last_error = Some(format!("HTTP error: {}", response.status()));
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Request failed: {}", e));
                }
            }
            
            if attempt < self._config.verus.max_retries {
                info!("RPC request failed, retrying... (attempt {}/{})", attempt + 1, self._config.verus.max_retries + 1);
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
            }
        }

        Err(crate::shared::error::AppError::Rpc(format!("RPC request failed after {} attempts: {:?}", self._config.verus.max_retries + 1, last_error)))
    }

    /// Check if external service is available
    pub async fn is_available(&self) -> bool {
        use reqwest::Client;
        use std::time::Duration;
        
        // Create HTTP client with short timeout for health check
        let client = match Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(client) => client,
            Err(_) => return false,
        };

        // Try to send a simple RPC request to check availability
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getinfo",
            "params": [],
            "id": "health_check"
        });

        match client
            .post(&self._config.verus.rpc_url)
            .header("Content-Type", "application/json")
            .basic_auth(&self._config.verus.rpc_user, Some(&self._config.verus.rpc_password))
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
} 