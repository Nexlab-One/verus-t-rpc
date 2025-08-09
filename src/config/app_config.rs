//! Application configuration structures
//! 
//! This module contains the main configuration structures for the application.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use validator::Validate;

/// Circuit breaker configuration for daemon connectivity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    #[validate(range(min = 1, max = 100))]
    pub failure_threshold: u32,
    
    /// Time to wait before testing recovery (seconds)
    #[validate(range(min = 1, max = 3600))]
    pub recovery_timeout_seconds: u64,
    
    /// Maximum requests allowed in half-open state
    #[validate(range(min = 1, max = 50))]
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_seconds: 60,
            half_open_max_requests: 3,
        }
    }
}

/// Verus daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerusConfig {
    /// RPC URL
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
    
    /// Circuit breaker configuration
    pub circuit_breaker: Option<CircuitBreakerConfig>,
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

/// PoW configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PowConfig {
    /// Default difficulty for PoW challenges
    #[validate(length(min = 1))]
    pub default_difficulty: String,
    
    /// Challenge expiration time in minutes
    #[validate(range(min = 1, max = 60))]
    pub challenge_expiration_minutes: u32,
    
    /// Token duration for PoW-validated tokens (seconds)
    #[validate(range(min = 3600, max = 86400))] // 1 hour to 24 hours
    pub token_duration_seconds: u64,
    
    /// Rate limit multiplier for PoW-validated tokens
    #[validate(range(min = 1.0, max = 10.0))]
    pub rate_limit_multiplier: f64,
    
    /// Enable PoW challenges
    pub enabled: bool,
}

/// Mining Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MiningPoolConfig {
    /// Pool server URL
    #[validate(url)]
    pub pool_url: String,
    
    /// Pool API key for authentication
    #[validate(length(min = 1))]
    pub api_key: String,
    
    /// Pool public key for signature verification
    #[validate(length(min = 1))]
    pub public_key: String,
    
    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: u64,
    
    /// Maximum retry attempts
    #[validate(range(min = 0, max = 10))]
    pub max_retries: u32,
    
    /// Circuit breaker threshold (number of failures before opening)
    #[validate(range(min = 1, max = 100))]
    pub circuit_breaker_threshold: u32,
    
    /// Circuit breaker timeout in seconds
    #[validate(range(min = 1, max = 3600))]
    pub circuit_breaker_timeout: u64,
    
    /// Rate limiting requests per minute
    #[validate(range(min = 1, max = 10000))]
    pub requests_per_minute: u32,
    
    /// Enable pool integration
    pub enabled: bool,
}

impl Default for MiningPoolConfig {
    fn default() -> Self {
        Self {
            pool_url: "https://pool.example.com".to_string(),
            api_key: "your-pool-api-key".to_string(),
            public_key: "pool-public-key".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            requests_per_minute: 100,
            enabled: false,
        }
    }
}

impl Default for PowConfig {
    fn default() -> Self {
        Self {
            default_difficulty: "0000ffff".to_string(),
            challenge_expiration_minutes: 10,
            token_duration_seconds: 14400, // 4 hours
            rate_limit_multiplier: 2.0,
            enabled: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    /// Allowed CORS origins (deprecated - use reverse proxy)
    pub cors_origins: Vec<String>,
    
    /// Allowed CORS methods (deprecated - use reverse proxy)
    pub cors_methods: Vec<String>,
    
    /// Allowed CORS headers (deprecated - use reverse proxy)
    pub cors_headers: Vec<String>,
    
    /// Enable request logging
    pub enable_request_logging: bool,
    
    /// Enable security headers
    pub enable_security_headers: bool,
    
    /// Trusted proxy headers
    pub trusted_proxy_headers: Vec<String>,
    
    /// Enable custom security headers
    pub enable_custom_headers: bool,
    
    /// Custom security header value
    pub custom_security_header: Option<String>,
    
    /// Method-specific rate limits
    pub method_rate_limits: std::collections::HashMap<String, RateLimitConfig>,
    
    /// JWT configuration
    pub jwt: JwtConfig,
    
    /// PoW configuration
    pub pow: Option<PowConfig>,
    
    /// Mining Pool configuration
    pub mining_pool: Option<MiningPoolConfig>,
    
    /// Development mode - allows local access without authentication
    pub development_mode: bool,
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

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct JwtConfig {
    /// JWT secret key
    #[validate(length(min = 32))]
    pub secret_key: String,
    
    /// JWT token expiration time in seconds
    #[validate(range(min = 60, max = 86400))] // 1 minute to 24 hours
    pub expiration_seconds: u64,
    
    /// JWT issuer
    #[validate(length(min = 1))]
    pub issuer: String,
    
    /// JWT audience
    #[validate(length(min = 1))]
    pub audience: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoggingConfig {
    /// Log level
    #[validate(length(min = 1))]
    pub level: String,
    
    /// Log format
    #[validate(length(min = 1))]
    pub format: String,
    
    /// Enable structured logging
    pub structured: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    
    /// Redis connection URL
    #[validate(url)]
    pub redis_url: String,
    
    /// Default TTL in seconds
    #[validate(range(min = 1, max = 86400))] // 1 second to 24 hours
    pub default_ttl: u64,
    
    /// Maximum cache size in bytes
    #[validate(range(min = 1024, max = 1073741824))] // 1KB to 1GB
    pub max_size: usize,
}

/// Payment tier configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PaymentTierConfig {
    #[validate(length(min = 1))]
    pub id: String,
    #[validate(range(min = 0.00000001))]
    pub amount_vrsc: f64,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// Payments configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PaymentsAppConfig {
    pub enabled: bool,
    /// Allowed shielded address types: "orchard", "sapling"
    pub address_types: Vec<String>,
    /// Default address type
    #[validate(length(min = 1))]
    pub default_address_type: String,
    /// Confirmations to issue provisional token
    #[validate(range(min = 0, max = 10))]
    pub min_confirmations: u32,
    /// Quote/session TTL in minutes
    #[validate(range(min = 1, max = 1440))]
    pub session_ttl_minutes: u32,
    /// Require viewing key presence to accept payments (view-only mode)
    pub require_viewing_key: bool,
    /// Optional list of viewing keys to import at startup
    pub viewing_keys: Vec<String>,
    /// Rescan mode for viewing key import: "yes", "no", or "whenkeyisnew"
    #[validate(length(min = 2))]
    pub viewing_key_rescan: String,
    /// Configured payment tiers
    pub tiers: Vec<PaymentTierConfig>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Verus daemon configuration
    pub verus: VerusConfig,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
    /// Payments configuration
    pub payments: PaymentsAppConfig,
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
                circuit_breaker: Some(CircuitBreakerConfig::default()),
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
                enable_custom_headers: false,
                custom_security_header: None,
                method_rate_limits: std::collections::HashMap::new(),
                jwt: JwtConfig {
                    secret_key: "your-super-secret-jwt-key-that-should-be-32-chars-min".to_string(),
                    expiration_seconds: 3600, // 1 hour
                    issuer: "verus-rpc-server".to_string(),
                    audience: "verus-clients".to_string(),
                },
                pow: None,
                mining_pool: None,
                development_mode: false,
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
            cache: CacheConfig::default(),
            payments: PaymentsAppConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_ttl: 300, // 5 minutes
            max_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl Default for PaymentsAppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address_types: vec!["orchard".to_string(), "sapling".to_string()],
            default_address_type: "orchard".to_string(),
            min_confirmations: 1,
            session_ttl_minutes: 30,
            require_viewing_key: false,
            viewing_keys: vec![],
            viewing_key_rescan: "whenkeyisnew".to_string(),
            tiers: vec![
                PaymentTierConfig {
                    id: "basic".to_string(),
                    amount_vrsc: 1.0,
                    description: Some("Basic access".to_string()),
                    permissions: vec!["read".to_string()],
                },
                PaymentTierConfig {
                    id: "pro".to_string(),
                    amount_vrsc: 5.0,
                    description: Some("Pro access".to_string()),
                    permissions: vec!["read".to_string(), "write".to_string()],
                },
            ],
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
            .map_err(|e| crate::shared::error::AppError::Config(format!("Failed to build configuration: {}", e)))?;
        
        let config: AppConfig = config.try_deserialize()
            .map_err(|e| crate::shared::error::AppError::Config(format!("Failed to deserialize configuration: {}", e)))?;
        
        // Validate configuration
        config.validate_config()
            .map_err(|e| crate::shared::error::AppError::Validation(format!("Configuration validation failed: {}", e)))?;
        
        Ok(config)
    }
    
    /// Validate the entire configuration
    pub fn validate_config(&self) -> Result<(), validator::ValidationErrors> {
        // Validate each section
        self.verus.validate()?;
        self.server.validate()?;
        self.security.validate()?;
        self.rate_limit.validate()?;
        self.logging.validate()?;
        self.cache.validate()?;
        // payments uses only simple validations; nothing extra
        
        Ok(())
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