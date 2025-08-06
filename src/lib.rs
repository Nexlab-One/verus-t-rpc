//! Verus RPC Server - A secure, high-performance RPC proxy for Verus blockchain
//! 
//! This library provides a secure HTTP API that acts as a proxy between clients
//! and the Verus daemon, with comprehensive security controls and validation.

// Configuration layer
pub mod config;

// Domain layer - Core business logic
pub mod domain;

// Application layer - Use cases and services
pub mod application;

// Infrastructure layer - External concerns and adapters
pub mod infrastructure;

// Shared utilities
pub mod shared;

// Middleware layer - Cross-cutting concerns
pub mod middleware;

// Re-export main types
pub use config::AppConfig;
pub use shared::error::{AppError, AppResult};
pub use infrastructure::http::server::HttpServer as VerusRpcServer;

/// Application result type
pub type Result<T> = std::result::Result<T, shared::error::AppError>; 