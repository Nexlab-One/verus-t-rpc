//! Security domain logic - Core security business rules and models

use crate::shared::error::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security policy for RPC methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Method-specific security rules
    pub method_rules: HashMap<String, MethodSecurityRule>,
    
    /// Default security settings
    pub default_rule: MethodSecurityRule,
    
    /// Global security settings
    pub global_settings: GlobalSecuritySettings,
}

/// Security rule for a specific method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSecurityRule {
    /// Whether authentication is required
    pub requires_auth: bool,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
    
    /// Rate limiting settings
    pub rate_limit: RateLimitSettings,
    
    /// Input validation rules
    pub validation_rules: Vec<ValidationRule>,
    
    /// Whether method is allowed
    pub allowed: bool,
}

/// Global security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSecuritySettings {
    /// Maximum request size in bytes
    pub max_request_size: usize,
    
    /// Maximum response size in bytes
    pub max_response_size: usize,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
    
    /// Whether to log security events
    pub log_security_events: bool,
    
    /// Allowed IP ranges
    pub allowed_ip_ranges: Vec<String>,
    
    /// Blocked IP ranges
    pub blocked_ip_ranges: Vec<String>,
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Requests per minute
    pub requests_per_minute: u32,
    
    /// Burst size
    pub burst_size: u32,
    
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

/// Input validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Maximum string length
    MaxLength(usize),
    
    /// Minimum string length
    MinLength(usize),
    
    /// Regular expression pattern
    Pattern(String),
    
    /// Numeric range
    NumericRange(f64, f64),
    
    /// Custom validation function
    Custom(String),
}

/// Security context for request processing
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Client IP address
    pub client_ip: String,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// User permissions
    pub user_permissions: Vec<String>,
    
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Request ID for tracking
    pub request_id: String,
    
    /// Development mode flag
    pub development_mode: bool,
}

/// Security event for logging and monitoring
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// Type of security event
    pub event_type: String,
    
    /// Client IP address
    pub client_ip: String,
    
    /// RPC method name
    pub method: String,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Additional event details
    pub details: String,
}

/// Security validator
pub struct SecurityValidator {
    policy: SecurityPolicy,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(policy: SecurityPolicy) -> Self {
        Self { policy }
    }
    
    /// Validate a request against security policy
    pub fn validate_request(&self, method: &str, context: &SecurityContext) -> AppResult<()> {
        // Get method-specific rule or use default
        let rule = self.policy.method_rules.get(method)
            .unwrap_or(&self.policy.default_rule);
        
        // Check if method is allowed
        if !rule.allowed {
            return Err(crate::shared::error::AppError::MethodNotAllowed {
                method: method.to_string(),
            });
        }
        
        // In development mode, skip authentication and permission checks for localhost
        if context.development_mode && self.is_localhost(&context.client_ip) {
            // Skip authentication and permission checks for local development
            return Ok(());
        }
        
        // Check authentication requirements
        if rule.requires_auth && context.auth_token.is_none() {
            return Err(crate::shared::error::AppError::Authentication("Authentication required".to_string()));
        }
        
        // Check permissions
        if !self.has_required_permissions(&rule.required_permissions, &context.user_permissions) {
            return Err(crate::shared::error::AppError::Security("Insufficient permissions".to_string()));
        }
        
        // Check IP restrictions
        self.validate_ip_address(&context.client_ip)?;
        
        Ok(())
    }
    
    /// Check if user has required permissions
    fn has_required_permissions(&self, required: &[String], user_permissions: &[String]) -> bool {
        if required.is_empty() {
            return true;
        }
        
        for permission in required {
            if !user_permissions.contains(permission) {
                return false;
            }
        }
        
        true
    }
    
    /// Validate IP address against allowed/blocked ranges
    fn validate_ip_address(&self, ip: &str) -> AppResult<()> {
        // Check blocked ranges first
        for blocked_range in &self.policy.global_settings.blocked_ip_ranges {
            if self.ip_matches_range(ip, blocked_range) {
                return Err(crate::shared::error::AppError::Security("IP address is blocked".to_string()));
            }
        }
        
        // Check allowed ranges if specified
        if !self.policy.global_settings.allowed_ip_ranges.is_empty() {
            let mut allowed = false;
            for allowed_range in &self.policy.global_settings.allowed_ip_ranges {
                if self.ip_matches_range(ip, allowed_range) {
                    allowed = true;
                    break;
                }
            }
            
            if !allowed {
                return Err(crate::shared::error::AppError::Security("IP address not allowed".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Check if IP matches a range (simplified implementation)
    fn ip_matches_range(&self, ip: &str, range: &str) -> bool {
        // Simplified IP range matching - in production, use a proper IP library
        if range == "*" {
            return true;
        }
        
        if range.ends_with("*") {
            let prefix = &range[..range.len() - 1];
            return ip.starts_with(prefix);
        }
        
        ip == range
    }
    
    /// Get rate limit settings for a method
    pub fn get_rate_limit_settings(&self, method: &str) -> &RateLimitSettings {
        let rule = self.policy.method_rules.get(method)
            .unwrap_or(&self.policy.default_rule);
        
        &rule.rate_limit
    }
    
    /// Get validation rules for a method
    pub fn get_validation_rules(&self, method: &str) -> &[ValidationRule] {
        let rule = self.policy.method_rules.get(method)
            .unwrap_or(&self.policy.default_rule);
        
        &rule.validation_rules
    }
    
    /// Check if IP address is localhost
    fn is_localhost(&self, ip: &str) -> bool {
        ip == "127.0.0.1" || ip == "::1" || ip == "localhost"
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            method_rules: HashMap::new(),
            default_rule: MethodSecurityRule {
                requires_auth: false,
                required_permissions: vec![],
                rate_limit: RateLimitSettings {
                    requests_per_minute: 1000,
                    burst_size: 100,
                    enabled: true,
                },
                validation_rules: vec![],
                allowed: true,
            },
            global_settings: GlobalSecuritySettings {
                max_request_size: 1024 * 1024, // 1MB
                max_response_size: 10 * 1024 * 1024, // 10MB
                session_timeout: 3600, // 1 hour
                log_security_events: true,
                allowed_ip_ranges: vec!["*".to_string()],
                blocked_ip_ranges: vec![],
            },
        }
    }
} 