//! RPC service that orchestrates RPC operations

use crate::{
    config::AppConfig,
    domain::{rpc::*, security::*},
    infrastructure::adapters::ComprehensiveValidator,
    shared::error::AppResult,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

/// RPC service that orchestrates RPC operations
pub struct RpcService {
    _config: Arc<AppConfig>,
    security_validator: Arc<SecurityValidator>,
    external_rpc_adapter: Arc<crate::infrastructure::adapters::ExternalRpcAdapter>,
    auth_adapter: Arc<crate::infrastructure::adapters::AuthenticationAdapter>,
    comprehensive_validator: Arc<ComprehensiveValidator>,
}

impl RpcService {
    /// Create a new RPC service
    pub fn new(config: Arc<AppConfig>, security_validator: Arc<SecurityValidator>) -> Self {
        let external_rpc_adapter = Arc::new(crate::infrastructure::adapters::ExternalRpcAdapter::new(config.clone()));
        let auth_adapter = Arc::new(crate::infrastructure::adapters::AuthenticationAdapter::new(config.clone()));
        let comprehensive_validator = Arc::new(ComprehensiveValidator::new());
        Self {
            _config: config,
            security_validator,
            external_rpc_adapter,
            auth_adapter,
            comprehensive_validator,
        }
    }

    /// Create a new RPC service with injected dependencies (for testing/DI)
    pub fn new_with_dependencies(
        config: Arc<AppConfig>,
        security_validator: Arc<SecurityValidator>,
        external_rpc_adapter: Arc<crate::infrastructure::adapters::ExternalRpcAdapter>,
        auth_adapter: Arc<crate::infrastructure::adapters::AuthenticationAdapter>,
        comprehensive_validator: Arc<ComprehensiveValidator>,
    ) -> Self {
        Self {
            _config: config,
            security_validator,
            external_rpc_adapter,
            auth_adapter,
            comprehensive_validator,
        }
    }

    /// Process an RPC request
    pub async fn process_request(&self, request: RpcRequest) -> AppResult<RpcResponse> {
        let auth_token: Option<String> = request.client_info.auth_token.clone();
        
        // Validate token and get user permissions
        let user_permissions = if let Some(token) = &auth_token {
            match self.auth_adapter.validate_token(token).await {
                Ok(permissions) => permissions,
                Err(e) => {
                    warn!("Authentication failed: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        };

        // Create security context
        let security_context = SecurityContext {
            client_ip: request.client_info.ip_address.clone(),
            user_agent: request.client_info.user_agent.clone(),
            auth_token,
            user_permissions,
            timestamp: request.client_info.timestamp,
            request_id: request.client_info.timestamp.timestamp_millis().to_string(),
            development_mode: self._config.security.development_mode,
        };

        // Validate request against security policy
        self.security_validator.validate_request(&request.method, &security_context)?;

        // Validate request structure
        request.validate()?;

        // Validate method parameters using comprehensive validator
        self.comprehensive_validator.validate_method(&request.method, &request.parameters)?;

        // Send request to external RPC service
        let response = self.external_rpc_adapter.send_request(&request).await?;

        info!(
            request_id = %security_context.request_id,
            method = %request.method,
            "RPC request processed successfully"
        );

        Ok(response)
    }

    /// Get method information
    pub fn get_method_info(&self, method_name: &str) -> Option<RpcMethod> {
        crate::application::services::rpc::method_registry::get_method_info(method_name)
    }

    /// Validate method parameters
    pub fn validate_method_parameters(&self, method: &str, parameters: &Value) -> AppResult<()> {
        if let Some(method_info) = self.get_method_info(method) {
            for rule in &method_info.parameter_rules {
                crate::application::services::rpc::parameter_validation::validate_parameter_rule(rule, parameters)?;
            }
        }

        Ok(())
    }

    // Parameter validation and token extraction moved to submodules
}