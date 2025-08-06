//! Application services - Orchestration of domain logic

use crate::{
    domain::{rpc::*, security::*},
    shared::error::AppResult,
    config::AppConfig,
    infrastructure::adapters::ComprehensiveValidator,
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

    /// Process an RPC request
    pub async fn process_request(&self, request: RpcRequest) -> AppResult<RpcResponse> {
        // Extract authentication token from request headers
        let auth_token = self.extract_auth_token_from_request(&request);
        
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
        // Method registry with security rules
        let method_registry = [
            ("getinfo", RpcMethod {
                name: "getinfo".to_string(),
                description: "Get blockchain information".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![],
            }),
            ("getblock", RpcMethod {
                name: "getblock".to_string(),
                description: "Get block information".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "hash".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(64)],
                    }
                ],
            }),
            ("sendrawtransaction", RpcMethod {
                name: "sendrawtransaction".to_string(),
                description: "Send raw transaction".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "hex".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(100)],
                    }
                ],
            }),
            ("makeOffer", RpcMethod {
                name: "makeOffer".to_string(),
                description: "Create marketplace offer".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "currency".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "offer".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Object,
                        required: true,
                        constraints: vec![],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 2,
                        name: "fromcurrency".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 3,
                        name: "tocurrency".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 4,
                        name: "amount".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 5,
                        name: "price".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 6,
                        name: "expiry".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                ],
            }),
            ("z_getnewaddress", RpcMethod {
                name: "z_getnewaddress".to_string(),
                description: "Get new Z-address".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "type".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::Custom("sprout|sapling|orchard".to_string())],
                    },
                ],
            }),
            ("z_listaddresses", RpcMethod {
                name: "z_listaddresses".to_string(),
                description: "List Z-addresses".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![],
            }),
            ("z_getbalance", RpcMethod {
                name: "z_getbalance".to_string(),
                description: "Get Z-address balance".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "address".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "minconf".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                ],
            }),
            ("z_sendmany", RpcMethod {
                name: "z_sendmany".to_string(),
                description: "Send to multiple Z-addresses".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "fromaddress".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "amounts".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Array,
                        required: true,
                        constraints: vec![],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 2,
                        name: "minconf".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 3,
                        name: "fee".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                ],
            }),
            ("z_shieldcoinbase", RpcMethod {
                name: "z_shieldcoinbase".to_string(),
                description: "Shield coinbase funds to Z-address".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "fromaddress".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "toaddress".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 2,
                        name: "fee".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 3,
                        name: "limit".to_string(),
                        param_type: crate::domain::rpc::ParameterType::Number,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::MinValue(0.0)],
                    },
                ],
            }),
            ("z_validateaddress", RpcMethod {
                name: "z_validateaddress".to_string(),
                description: "Validate Z-address".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "address".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                ],
            }),
            ("z_viewtransaction", RpcMethod {
                name: "z_viewtransaction".to_string(),
                description: "View Z-transaction details".to_string(),
                read_only: true,
                required_permissions: vec!["read".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "txid".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                ],
            }),
            ("z_exportkey", RpcMethod {
                name: "z_exportkey".to_string(),
                description: "Export Z-address private key".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "address".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                ],
            }),
            ("z_importkey", RpcMethod {
                name: "z_importkey".to_string(),
                description: "Import Z-address private key".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "zkey".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "rescan".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::Custom("yes|no|whenkeyisnew".to_string())],
                    },
                ],
            }),
            ("z_exportviewingkey", RpcMethod {
                name: "z_exportviewingkey".to_string(),
                description: "Export Z-address viewing key".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "address".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                ],
            }),
            ("z_importviewingkey", RpcMethod {
                name: "z_importviewingkey".to_string(),
                description: "Import Z-address viewing key".to_string(),
                read_only: false,
                required_permissions: vec!["write".to_string()],
                parameter_rules: vec![
                    crate::domain::rpc::ParameterRule {
                        index: 0,
                        name: "vkey".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: true,
                        constraints: vec![crate::domain::rpc::Constraint::MinLength(1)],
                    },
                    crate::domain::rpc::ParameterRule {
                        index: 1,
                        name: "rescan".to_string(),
                        param_type: crate::domain::rpc::ParameterType::String,
                        required: false,
                        constraints: vec![crate::domain::rpc::Constraint::Custom("yes|no|whenkeyisnew".to_string())],
                    },
                ],
            }),
        ];

        method_registry
            .iter()
            .find(|(name, _)| *name == method_name)
            .map(|(_, method)| method.clone())
    }

    /// Validate method parameters
    pub fn validate_method_parameters(&self, method: &str, parameters: &Value) -> AppResult<()> {
        if let Some(method_info) = self.get_method_info(method) {
            // Apply validation rules
            for rule in &method_info.parameter_rules {
                self.validate_parameter_rule(rule, parameters)?;
            }
        }

        Ok(())
    }

    /// Validate a single parameter rule
    fn validate_parameter_rule(&self, rule: &crate::domain::rpc::ParameterRule, parameters: &Value) -> AppResult<()> {
        // Get parameter value based on index
        let param_value = match parameters {
            Value::Array(arr) => {
                if rule.index < arr.len() {
                    Some(&arr[rule.index])
                } else if rule.required {
                    return Err(crate::shared::error::AppError::InvalidParameters {
                        method: rule.name.clone(),
                        reason: format!("Required parameter at index {} not found", rule.index),
                    });
                } else {
                    None
                }
            }
            Value::Object(obj) => {
                obj.get(&rule.name)
            }
            _ => {
                if rule.required {
                    return Err(crate::shared::error::AppError::InvalidParameters {
                        method: rule.name.clone(),
                        reason: "Parameters must be array or object".to_string(),
                    });
                } else {
                    None
                }
            }
        };

        // Validate parameter value if present
        if let Some(value) = param_value {
            self.validate_parameter_value(rule, value)?;
        }

        Ok(())
    }

    /// Validate parameter value against constraints
    fn validate_parameter_value(&self, rule: &crate::domain::rpc::ParameterRule, value: &Value) -> AppResult<()> {
        for constraint in &rule.constraints {
            match constraint {
                crate::domain::rpc::Constraint::MinLength(min_len) => {
                    if let Value::String(s) = value {
                        if s.len() < *min_len {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: rule.name.clone(),
                                reason: format!("Parameter {} too short (min {} characters)", rule.name, min_len),
                            });
                        }
                    }
                }
                crate::domain::rpc::Constraint::MaxLength(max_len) => {
                    if let Value::String(s) = value {
                        if s.len() > *max_len {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: rule.name.clone(),
                                reason: format!("Parameter {} too long (max {} characters)", rule.name, max_len),
                            });
                        }
                    }
                }
                crate::domain::rpc::Constraint::Pattern(pattern) => {
                    if let Value::String(s) = value {
                        use regex::Regex;
                        match Regex::new(pattern) {
                            Ok(regex) => {
                                if !regex.is_match(s) {
                                    return Err(crate::shared::error::AppError::InvalidParameters {
                                        method: rule.name.clone(),
                                        reason: format!("Parameter {} does not match pattern {}", rule.name, pattern),
                                    });
                                }
                            }
                            Err(e) => {
                                return Err(crate::shared::error::AppError::Validation(
                                    format!("Invalid regex pattern '{}': {}", pattern, e)
                                ));
                            }
                        }
                    }
                }
                _ => {
                    warn!("Constraint validation not yet implemented: {:?}", constraint);
                }
            }
        }

        Ok(())
    }

    /// Extract authentication token from request headers
    fn extract_auth_token_from_request(&self, request: &RpcRequest) -> Option<String> {
        // For now, we'll extract from the user_agent field as a placeholder
        // In a real implementation, this would extract from actual HTTP headers
        if let Some(user_agent) = &request.client_info.user_agent {
            if user_agent.contains("Bearer ") {
                // Extract token from user agent (temporary implementation)
                let parts: Vec<&str> = user_agent.split("Bearer ").collect();
                if parts.len() > 1 {
                    return Some(format!("Bearer {}", parts[1]));
                }
            }
        }
        None
    }
}

/// Metrics service for collecting application metrics
pub struct MetricsService {
    total_requests: std::sync::atomic::AtomicU64,
    successful_requests: std::sync::atomic::AtomicU64,
    failed_requests: std::sync::atomic::AtomicU64,
    rate_limited_requests: std::sync::atomic::AtomicU64,
    total_response_time: std::sync::atomic::AtomicU64,
    response_count: std::sync::atomic::AtomicU64,
    active_connections: std::sync::atomic::AtomicU32,
    start_time: std::time::Instant,
}

impl MetricsService {
    /// Create a new metrics service
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            successful_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            rate_limited_requests: std::sync::atomic::AtomicU64::new(0),
            total_response_time: std::sync::atomic::AtomicU64::new(0),
            response_count: std::sync::atomic::AtomicU64::new(0),
            active_connections: std::sync::atomic::AtomicU32::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&self, success: bool) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Record a rate limited request
    pub fn record_rate_limited_request(&self) {
        self.rate_limited_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record response time
    pub fn record_response_time(&self, duration_ms: u64) {
        self.total_response_time.fetch_add(duration_ms, std::sync::atomic::Ordering::Relaxed);
        self.response_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn increment_active_connections(&self) {
        self.active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn decrement_active_connections(&self) {
        self.active_connections.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Value {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let successful = self.successful_requests.load(std::sync::atomic::Ordering::Relaxed);
        let failed = self.failed_requests.load(std::sync::atomic::Ordering::Relaxed);
        let rate_limited = self.rate_limited_requests.load(std::sync::atomic::Ordering::Relaxed);
        let total_response_time = self.total_response_time.load(std::sync::atomic::Ordering::Relaxed);
        let response_count = self.response_count.load(std::sync::atomic::Ordering::Relaxed);
        let active_connections = self.active_connections.load(std::sync::atomic::Ordering::Relaxed);
        let uptime = self.start_time.elapsed().as_secs();

        let avg_response_time_ms = if response_count > 0 {
            total_response_time as f64 / response_count as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_requests": total,
            "successful_requests": successful,
            "failed_requests": failed,
            "rate_limited_requests": rate_limited,
            "avg_response_time_ms": avg_response_time_ms,
            "active_connections": active_connections,
            "uptime_seconds": uptime,
        })
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
} 