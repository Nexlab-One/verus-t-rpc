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

    /// Validate if a method is allowed (without full context)
    pub fn validate_method(&self, method: &str) -> AppResult<()> {
        let rule = self.policy.method_rules.get(method)
            .unwrap_or(&self.policy.default_rule);
        
        if !rule.allowed {
            return Err(crate::shared::error::AppError::MethodNotAllowed {
                method: method.to_string(),
            });
        }
        
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_security_context() -> SecurityContext {
        SecurityContext {
            client_ip: "127.0.0.1".to_string(),
            user_agent: Some("test-agent".to_string()),
            auth_token: None,
            user_permissions: vec!["read".to_string()],
            timestamp: Utc::now(),
            request_id: "test-request-id".to_string(),
            development_mode: true,
        }
    }

    fn create_test_security_context_with_auth(auth_token: &str) -> SecurityContext {
        SecurityContext {
            client_ip: "127.0.0.1".to_string(),
            user_agent: Some("test-agent".to_string()),
            auth_token: Some(auth_token.to_string()),
            user_permissions: vec!["read".to_string(), "write".to_string()],
            timestamp: Utc::now(),
            request_id: "test-request-id".to_string(),
            development_mode: true,
        }
    }

    #[test]
    fn test_security_validator_new() {
        let policy = SecurityPolicy::default();
        let validator = SecurityValidator::new(policy);
        
        // Verify validator was created successfully
        assert!(validator.get_rate_limit_settings("getinfo").enabled);
    }

    #[test]
    fn test_security_validator_validate_method() {
        let policy = SecurityPolicy::default();
        let validator = SecurityValidator::new(policy);
        
        // Test valid method (default policy allows all)
        let result = validator.validate_method("getinfo");
        assert!(result.is_ok());
        
        let result = validator.validate_method("getblock");
        assert!(result.is_ok());
        
        let result = validator.validate_method("sendrawtransaction");
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_validator_validate_method_disallowed() {
        let mut policy = SecurityPolicy::default();
        
        // Create a method rule that disallows a specific method
        let method_rule = MethodSecurityRule {
            requires_auth: false,
            required_permissions: vec![],
            rate_limit: RateLimitSettings {
                requests_per_minute: 100,
                burst_size: 10,
                enabled: true,
            },
            validation_rules: vec![],
            allowed: false, // Disallow this method
        };
        
        policy.method_rules.insert("disallowed_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test disallowed method
        let result = validator.validate_method("disallowed_method");
        assert!(result.is_err());
        
        // Test that other methods are still allowed
        let result = validator.validate_method("getinfo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_validator_validate_request() {
        let policy = SecurityPolicy::default();
        let validator = SecurityValidator::new(policy);
        let context = create_test_security_context();
        
        // Test valid request
        let result = validator.validate_request("getinfo", &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_validator_validate_request_requires_auth() {
        let mut policy = SecurityPolicy::default();
        
        // Create a method rule that requires authentication
        let method_rule = MethodSecurityRule {
            requires_auth: true,
            required_permissions: vec!["read".to_string()],
            rate_limit: RateLimitSettings {
                requests_per_minute: 100,
                burst_size: 10,
                enabled: true,
            },
            validation_rules: vec![],
            allowed: true,
        };
        
        policy.method_rules.insert("auth_required_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test without auth token (should fail)
        let mut context = create_test_security_context();
        context.development_mode = false; // Disable development mode to test auth requirement
        let result = validator.validate_request("auth_required_method", &context);
        assert!(result.is_err());
        
        // Test with auth token (should succeed)
        let context = create_test_security_context_with_auth("test-token");
        let result = validator.validate_request("auth_required_method", &context);
        assert!(result.is_ok());
        
        // Test default method (should succeed without auth due to default policy)
        let result = validator.validate_request("getinfo", &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_validator_validate_request_insufficient_permissions() {
        let mut policy = SecurityPolicy::default();
        
        // Create a method rule that requires specific permissions
        let method_rule = MethodSecurityRule {
            requires_auth: true,
            required_permissions: vec!["admin".to_string()], // Require admin permission
            rate_limit: RateLimitSettings {
                requests_per_minute: 100,
                burst_size: 10,
                enabled: true,
            },
            validation_rules: vec![],
            allowed: true,
        };
        
        policy.method_rules.insert("admin_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test with insufficient permissions
        let mut context = create_test_security_context_with_auth("test-token");
        context.development_mode = false; // Disable development mode to test auth requirement
        let result = validator.validate_request("admin_method", &context);
        assert!(result.is_err());
        
        // Test with sufficient permissions
        let mut context = create_test_security_context_with_auth("test-token");
        context.user_permissions = vec!["read".to_string(), "write".to_string(), "admin".to_string()];
        let result = validator.validate_request("admin_method", &context);
        assert!(result.is_ok());
        
        // Test default method (should succeed with any permissions due to default policy)
        let context = create_test_security_context_with_auth("test-token");
        let result = validator.validate_request("getinfo", &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_validator_validate_request_development_mode() {
        let mut policy = SecurityPolicy::default();
        
        // Create a method rule that requires authentication
        let method_rule = MethodSecurityRule {
            requires_auth: true,
            required_permissions: vec!["read".to_string()],
            rate_limit: RateLimitSettings {
                requests_per_minute: 100,
                burst_size: 10,
                enabled: true,
            },
            validation_rules: vec![],
            allowed: true,
        };
        
        policy.method_rules.insert("auth_required_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test in development mode with localhost (should bypass auth)
        let mut context = create_test_security_context();
        context.development_mode = true;
        context.client_ip = "127.0.0.1".to_string();
        
        let result = validator.validate_request("auth_required_method", &context);
        assert!(result.is_ok());
        
        // Test in development mode with non-localhost (should still require auth)
        context.client_ip = "192.168.1.1".to_string();
        let result = validator.validate_request("auth_required_method", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_security_validator_get_rate_limit_settings() {
        let mut policy = SecurityPolicy::default();
        
        // Create a custom rate limit for a specific method
        let custom_rate_limit = RateLimitSettings {
            requests_per_minute: 50,
            burst_size: 5,
            enabled: true,
        };
        
        let method_rule = MethodSecurityRule {
            requires_auth: false,
            required_permissions: vec![],
            rate_limit: custom_rate_limit,
            validation_rules: vec![],
            allowed: true,
        };
        
        policy.method_rules.insert("rate_limited_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test custom rate limit
        let rate_limit = validator.get_rate_limit_settings("rate_limited_method");
        assert_eq!(rate_limit.requests_per_minute, 50);
        assert_eq!(rate_limit.burst_size, 5);
        assert!(rate_limit.enabled);
        
        // Test default rate limit for unknown method
        let rate_limit = validator.get_rate_limit_settings("unknown_method");
        assert_eq!(rate_limit.requests_per_minute, 1000); // Default value
        assert_eq!(rate_limit.burst_size, 100); // Default value
        assert!(rate_limit.enabled);
    }

    #[test]
    fn test_security_validator_get_validation_rules() {
        let mut policy = SecurityPolicy::default();
        
        // Create validation rules for a specific method
        let validation_rules = vec![
            ValidationRule::MaxLength(100),
            ValidationRule::Pattern(r"^[a-zA-Z0-9]+$".to_string()),
        ];
        
        let method_rule = MethodSecurityRule {
            requires_auth: false,
            required_permissions: vec![],
            rate_limit: RateLimitSettings {
                requests_per_minute: 100,
                burst_size: 10,
                enabled: true,
            },
            validation_rules: validation_rules.clone(),
            allowed: true,
        };
        
        policy.method_rules.insert("validated_method".to_string(), method_rule);
        let validator = SecurityValidator::new(policy);
        
        // Test validation rules
        let rules = validator.get_validation_rules("validated_method");
        assert_eq!(rules.len(), 2);
        
        // Test default validation rules for unknown method
        let rules = validator.get_validation_rules("unknown_method");
        assert_eq!(rules.len(), 0); // Default is empty
    }

    #[test]
    fn test_security_validator_ip_address_validation() {
        let mut policy = SecurityPolicy::default();
        
        // Set up IP restrictions
        policy.global_settings.blocked_ip_ranges = vec!["192.168.1.0/24".to_string()];
        policy.global_settings.allowed_ip_ranges = vec!["127.0.0.1".to_string(), "10.0.0.0/8".to_string()];
        
        let validator = SecurityValidator::new(policy);
        let context = create_test_security_context();
        
        // Test allowed IP
        let result = validator.validate_request("getinfo", &context);
        assert!(result.is_ok());
        
        // Test blocked IP
        let mut blocked_context = create_test_security_context();
        blocked_context.client_ip = "192.168.1.100".to_string();
        let result = validator.validate_request("getinfo", &blocked_context);
        assert!(result.is_err());
        
        // Test IP not in allowed range
        let mut disallowed_context = create_test_security_context();
        disallowed_context.client_ip = "172.16.0.1".to_string();
        let result = validator.validate_request("getinfo", &disallowed_context);
        assert!(result.is_err());
    }

    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        
        // Test default values
        assert!(policy.default_rule.allowed);
        assert!(!policy.default_rule.requires_auth);
        assert!(policy.default_rule.required_permissions.is_empty());
        assert!(policy.default_rule.validation_rules.is_empty());
        
        assert_eq!(policy.global_settings.max_request_size, 1024 * 1024);
        assert_eq!(policy.global_settings.max_response_size, 10 * 1024 * 1024);
        assert_eq!(policy.global_settings.session_timeout, 3600);
        assert!(policy.global_settings.log_security_events);
        assert_eq!(policy.global_settings.allowed_ip_ranges, vec!["*".to_string()]);
        assert!(policy.global_settings.blocked_ip_ranges.is_empty());
    }

    #[test]
    fn test_security_context_creation() {
        let context = create_test_security_context();
        
        assert_eq!(context.client_ip, "127.0.0.1");
        assert_eq!(context.user_agent, Some("test-agent".to_string()));
        assert_eq!(context.auth_token, None);
        assert_eq!(context.user_permissions, vec!["read".to_string()]);
        assert_eq!(context.request_id, "test-request-id");
        assert!(context.development_mode);
    }

    #[test]
    fn test_security_context_with_auth() {
        let context = create_test_security_context_with_auth("test-token");
        
        assert_eq!(context.auth_token, Some("test-token".to_string()));
        assert_eq!(context.user_permissions, vec!["read".to_string(), "write".to_string()]);
    }
} 