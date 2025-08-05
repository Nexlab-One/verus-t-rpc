use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use validator::Validate;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    /// Verus daemon RPC configuration
    #[validate(nested)]
    pub verus: VerusConfig,
    
    /// Server configuration
    #[validate(nested)]
    pub server: ServerConfig,
    
    /// Security configuration
    #[validate(nested)]
    pub security: SecurityConfig,
    
    /// Rate limiting configuration
    #[validate(nested)]
    pub rate_limit: RateLimitConfig,
    
    /// Logging configuration
    #[validate(nested)]
    pub logging: LoggingConfig,
}

/// Verus daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerusConfig {
    /// RPC URL (e.g., "http://127.0.0.1:27486")
    #[validate(url)]
    pub rpc_url: String,
    
    /// RPC username
    #[validate(length(min = 1))]
    pub rpc_user: String,
    
    /// RPC password
    #[validate(length(min = 1))]
    pub rpc_password: String,
    
    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: u64,
    
    /// Maximum retry attempts
    #[validate(range(min = 0, max = 10))]
    pub max_retries: u32,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    /// Server address to bind to
    pub bind_address: IpAddr,
    
    /// Server port
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    
    /// Maximum request size in bytes
    #[validate(range(min = 1024, max = 10485760))] // 1KB to 10MB
    pub max_request_size: usize,
    
    /// Worker threads (0 for auto-detect)
    #[validate(range(min = 0, max = 64))]
    pub worker_threads: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    
    /// Allowed CORS methods
    pub cors_methods: Vec<String>,
    
    /// Allowed CORS headers
    pub cors_headers: Vec<String>,
    
    /// Enable request logging
    pub enable_request_logging: bool,
    
    /// Enable security headers
    pub enable_security_headers: bool,
    
    /// Trusted proxy headers
    pub trusted_proxy_headers: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    /// Requests per minute per IP
    #[validate(range(min = 1, max = 10000))]
    pub requests_per_minute: u32,
    
    /// Burst size
    #[validate(range(min = 1, max = 1000))]
    pub burst_size: u32,
    
    /// Enable rate limiting
    pub enabled: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (json, text)
    pub format: String,
    
    /// Enable structured logging
    pub structured: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            verus: VerusConfig {
                rpc_url: "http://127.0.0.1:27486".to_string(),
                rpc_user: "rpcuser".to_string(),
                rpc_password: "rpcpassword".to_string(),
                timeout_seconds: 30,
                max_retries: 3,
            },
            server: ServerConfig {
                bind_address: "127.0.0.1".parse().unwrap(),
                port: 8080,
                max_request_size: 1024 * 1024, // 1MB
                worker_threads: 0, // Auto-detect
            },
            security: SecurityConfig {
                cors_origins: vec!["*".to_string()],
                cors_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
                cors_headers: vec![
                    "Content-Type".to_string(),
                    "Authorization".to_string(),
                    "Accept".to_string(),
                ],
                enable_request_logging: true,
                enable_security_headers: true,
                trusted_proxy_headers: vec!["X-Forwarded-For".to_string()],
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: 1000,
                burst_size: 100,
                enabled: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                structured: true,
            },
        }
    }
}

impl AppConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> crate::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("Conf").required(false))
            .add_source(config::Environment::with_prefix("VERUS_RPC").separator("__"))
            .build()
            .map_err(|e| crate::error::AppError::Config(format!("Failed to build configuration: {}", e)))?;
        
        let config: AppConfig = config.try_deserialize()
            .map_err(|e| crate::error::AppError::Config(format!("Failed to deserialize configuration: {}", e)))?;
        
        // Validate configuration
        config.validate()
            .map_err(|e| crate::error::AppError::Validation(format!("Configuration validation failed: {}", e)))?;
        
        Ok(config)
    }
    
    /// Get server address as string
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.bind_address, self.server.port)
    }
    
    /// Check if CORS is configured for any origin
    pub fn cors_allow_any_origin(&self) -> bool {
        self.security.cors_origins.contains(&"*".to_string())
    }
} 