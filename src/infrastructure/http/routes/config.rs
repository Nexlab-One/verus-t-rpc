//! Route configuration structures module
//! 
//! This module contains structured configuration objects for route settings
//! to provide better organization and type safety for route configuration.

use crate::config::AppConfig;

/// Route configuration for RPC endpoints
#[derive(Debug, Clone)]
pub struct RpcRouteConfig {
    /// Maximum request size in bytes
    pub max_request_size: u64,
    /// Whether to enable request logging
    pub enable_logging: bool,
    /// Required headers for the route
    pub required_headers: Vec<String>,
    /// HTTP methods allowed
    pub allowed_methods: Vec<String>,
}

impl RpcRouteConfig {
    /// Create RPC route configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            max_request_size: config.server.max_request_size as u64,
            enable_logging: config.security.enable_request_logging,
            required_headers: vec!["x-forwarded-for".to_string()],
            allowed_methods: vec!["POST".to_string()],
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_request_size == 0 {
            return Err("Max request size must be greater than 0".to_string());
        }
        if self.allowed_methods.is_empty() {
            return Err("At least one HTTP method must be allowed".to_string());
        }
        Ok(())
    }
}

/// Route configuration for health check endpoints
#[derive(Debug, Clone)]
pub struct HealthRouteConfig {
    /// Whether to enable request logging
    pub enable_logging: bool,
    /// HTTP methods allowed
    pub allowed_methods: Vec<String>,
}

impl HealthRouteConfig {
    /// Create health route configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            enable_logging: config.security.enable_request_logging,
            allowed_methods: vec!["GET".to_string()],
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.allowed_methods.is_empty() {
            return Err("At least one HTTP method must be allowed".to_string());
        }
        Ok(())
    }
}

/// Route configuration for metrics endpoints
#[derive(Debug, Clone)]
pub struct MetricsRouteConfig {
    /// Whether to enable request logging
    pub enable_logging: bool,
    /// HTTP methods allowed
    pub allowed_methods: Vec<String>,
    /// Whether to include detailed metrics
    pub include_detailed_metrics: bool,
}

impl MetricsRouteConfig {
    /// Create metrics route configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            enable_logging: config.security.enable_request_logging,
            allowed_methods: vec!["GET".to_string()],
            include_detailed_metrics: true, // Default to true for detailed metrics
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.allowed_methods.is_empty() {
            return Err("At least one HTTP method must be allowed".to_string());
        }
        Ok(())
    }
}

/// Route configuration for mining pool endpoints
#[derive(Debug, Clone)]
pub struct MiningPoolRouteConfig {
    /// Maximum request size in bytes
    pub max_request_size: u64,
    /// Whether to enable request logging
    pub enable_logging: bool,
    /// Required headers for the route
    pub required_headers: Vec<String>,
    /// HTTP methods allowed for share validation
    pub share_validation_methods: Vec<String>,
    /// HTTP methods allowed for metrics
    pub metrics_methods: Vec<String>,
}

impl MiningPoolRouteConfig {
    /// Create mining pool route configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            max_request_size: config.server.max_request_size as u64,
            enable_logging: config.security.enable_request_logging,
            required_headers: vec!["x-forwarded-for".to_string()],
            share_validation_methods: vec!["POST".to_string()],
            metrics_methods: vec!["GET".to_string()],
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_request_size == 0 {
            return Err("Max request size must be greater than 0".to_string());
        }
        if self.share_validation_methods.is_empty() {
            return Err("At least one HTTP method must be allowed for share validation".to_string());
        }
        if self.metrics_methods.is_empty() {
            return Err("At least one HTTP method must be allowed for metrics".to_string());
        }
        Ok(())
    }
}

/// Global route configuration that contains all route-specific configs
#[derive(Debug, Clone)]
pub struct RouteConfig {
    /// RPC route configuration
    pub rpc: RpcRouteConfig,
    /// Health route configuration
    pub health: HealthRouteConfig,
    /// Metrics route configuration
    pub metrics: MetricsRouteConfig,
    /// Mining pool route configuration
    pub mining_pool: MiningPoolRouteConfig,
}

impl RouteConfig {
    /// Create route configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            rpc: RpcRouteConfig::from_app_config(config),
            health: HealthRouteConfig::from_app_config(config),
            metrics: MetricsRouteConfig::from_app_config(config),
            mining_pool: MiningPoolRouteConfig::from_app_config(config),
        }
    }

    /// Validate all route configurations
    pub fn validate(&self) -> Result<(), String> {
        self.rpc.validate()?;
        self.health.validate()?;
        self.metrics.validate()?;
        self.mining_pool.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn test_rpc_route_config_creation() {
        let config = create_test_config();
        let rpc_config = RpcRouteConfig::from_app_config(&config);

        assert_eq!(rpc_config.max_request_size, config.server.max_request_size as u64);
        assert_eq!(rpc_config.enable_logging, config.security.enable_request_logging);
        assert_eq!(rpc_config.required_headers, vec!["x-forwarded-for".to_string()]);
        assert_eq!(rpc_config.allowed_methods, vec!["POST".to_string()]);
    }

    #[test]
    fn test_rpc_route_config_validation_success() {
        let config = create_test_config();
        let rpc_config = RpcRouteConfig::from_app_config(&config);
        
        assert!(rpc_config.validate().is_ok());
    }

    #[test]
    fn test_rpc_route_config_validation_zero_max_size() {
        let rpc_config = RpcRouteConfig {
            max_request_size: 0,
            enable_logging: true,
            required_headers: vec!["x-forwarded-for".to_string()],
            allowed_methods: vec!["POST".to_string()],
        };
        
        assert!(rpc_config.validate().is_err());
        assert!(rpc_config.validate().unwrap_err().contains("Max request size must be greater than 0"));
    }

    #[test]
    fn test_rpc_route_config_validation_empty_methods() {
        let rpc_config = RpcRouteConfig {
            max_request_size: 1024,
            enable_logging: true,
            required_headers: vec!["x-forwarded-for".to_string()],
            allowed_methods: vec![],
        };
        
        assert!(rpc_config.validate().is_err());
        assert!(rpc_config.validate().unwrap_err().contains("At least one HTTP method must be allowed"));
    }

    #[test]
    fn test_health_route_config_creation() {
        let config = create_test_config();
        let health_config = HealthRouteConfig::from_app_config(&config);

        assert_eq!(health_config.enable_logging, config.security.enable_request_logging);
        assert_eq!(health_config.allowed_methods, vec!["GET".to_string()]);
    }

    #[test]
    fn test_health_route_config_validation_success() {
        let config = create_test_config();
        let health_config = HealthRouteConfig::from_app_config(&config);
        
        assert!(health_config.validate().is_ok());
    }

    #[test]
    fn test_health_route_config_validation_empty_methods() {
        let health_config = HealthRouteConfig {
            enable_logging: true,
            allowed_methods: vec![],
        };
        
        assert!(health_config.validate().is_err());
        assert!(health_config.validate().unwrap_err().contains("At least one HTTP method must be allowed"));
    }

    #[test]
    fn test_metrics_route_config_creation() {
        let config = create_test_config();
        let metrics_config = MetricsRouteConfig::from_app_config(&config);

        assert_eq!(metrics_config.enable_logging, config.security.enable_request_logging);
        assert_eq!(metrics_config.allowed_methods, vec!["GET".to_string()]);
        assert!(metrics_config.include_detailed_metrics);
    }

    #[test]
    fn test_metrics_route_config_validation_success() {
        let config = create_test_config();
        let metrics_config = MetricsRouteConfig::from_app_config(&config);
        
        assert!(metrics_config.validate().is_ok());
    }

    #[test]
    fn test_metrics_route_config_validation_empty_methods() {
        let metrics_config = MetricsRouteConfig {
            enable_logging: true,
            allowed_methods: vec![],
            include_detailed_metrics: true,
        };
        
        assert!(metrics_config.validate().is_err());
        assert!(metrics_config.validate().unwrap_err().contains("At least one HTTP method must be allowed"));
    }

    #[test]
    fn test_mining_pool_route_config_creation() {
        let config = create_test_config();
        let mining_pool_config = MiningPoolRouteConfig::from_app_config(&config);

        assert_eq!(mining_pool_config.max_request_size, config.server.max_request_size as u64);
        assert_eq!(mining_pool_config.enable_logging, config.security.enable_request_logging);
        assert_eq!(mining_pool_config.required_headers, vec!["x-forwarded-for".to_string()]);
        assert_eq!(mining_pool_config.share_validation_methods, vec!["POST".to_string()]);
        assert_eq!(mining_pool_config.metrics_methods, vec!["GET".to_string()]);
    }

    #[test]
    fn test_mining_pool_route_config_validation_success() {
        let config = create_test_config();
        let mining_pool_config = MiningPoolRouteConfig::from_app_config(&config);
        
        assert!(mining_pool_config.validate().is_ok());
    }

    #[test]
    fn test_mining_pool_route_config_validation_zero_max_size() {
        let mining_pool_config = MiningPoolRouteConfig {
            max_request_size: 0,
            enable_logging: true,
            required_headers: vec!["x-forwarded-for".to_string()],
            share_validation_methods: vec!["POST".to_string()],
            metrics_methods: vec!["GET".to_string()],
        };
        
        assert!(mining_pool_config.validate().is_err());
        assert!(mining_pool_config.validate().unwrap_err().contains("Max request size must be greater than 0"));
    }

    #[test]
    fn test_mining_pool_route_config_validation_empty_share_methods() {
        let mining_pool_config = MiningPoolRouteConfig {
            max_request_size: 1024,
            enable_logging: true,
            required_headers: vec!["x-forwarded-for".to_string()],
            share_validation_methods: vec![],
            metrics_methods: vec!["GET".to_string()],
        };
        
        assert!(mining_pool_config.validate().is_err());
        assert!(mining_pool_config.validate().unwrap_err().contains("At least one HTTP method must be allowed for share validation"));
    }

    #[test]
    fn test_mining_pool_route_config_validation_empty_metrics_methods() {
        let mining_pool_config = MiningPoolRouteConfig {
            max_request_size: 1024,
            enable_logging: true,
            required_headers: vec!["x-forwarded-for".to_string()],
            share_validation_methods: vec!["POST".to_string()],
            metrics_methods: vec![],
        };
        
        assert!(mining_pool_config.validate().is_err());
        assert!(mining_pool_config.validate().unwrap_err().contains("At least one HTTP method must be allowed for metrics"));
    }

    #[test]
    fn test_route_config_creation() {
        let config = create_test_config();
        let route_config = RouteConfig::from_app_config(&config);

        assert_eq!(route_config.rpc.max_request_size, config.server.max_request_size as u64);
        assert_eq!(route_config.health.allowed_methods, vec!["GET".to_string()]);
        assert_eq!(route_config.metrics.allowed_methods, vec!["GET".to_string()]);
        assert_eq!(route_config.mining_pool.share_validation_methods, vec!["POST".to_string()]);
    }

    #[test]
    fn test_route_config_validation_success() {
        let config = create_test_config();
        let route_config = RouteConfig::from_app_config(&config);
        
        assert!(route_config.validate().is_ok());
    }

    #[test]
    fn test_route_config_validation_failure_propagates() {
        let mut route_config = RouteConfig::from_app_config(&create_test_config());
        route_config.rpc.max_request_size = 0; // This will cause validation to fail
        
        assert!(route_config.validate().is_err());
        assert!(route_config.validate().unwrap_err().contains("Max request size must be greater than 0"));
    }
}
