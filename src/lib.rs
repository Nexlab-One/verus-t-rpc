//! Verus RPC Server - A secure, high-performance RPC proxy for Verus blockchain
//! 
//! This library provides a secure HTTP API that acts as a proxy between clients
//! and the Verus daemon, with comprehensive security controls and validation.

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod rpc;
pub mod security;
pub mod server;
pub mod validation;

pub use config::AppConfig;
pub use error::{AppError, AppResult};
pub use server::VerusRpcServer;

/// Application result type
pub type Result<T> = std::result::Result<T, error::AppError>; 