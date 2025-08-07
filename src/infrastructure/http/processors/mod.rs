//! HTTP request processors module
//! 
//! This module contains common request processing patterns that are shared
//! across different endpoint handlers.

pub mod base;
pub mod rpc;

pub use base::BaseRequestProcessor;
pub use rpc::RpcRequestProcessor;
