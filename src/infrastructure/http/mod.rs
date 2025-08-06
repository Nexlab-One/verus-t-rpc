//! HTTP infrastructure module
//! 
//! This module contains HTTP-related concerns including models,
//! server implementation, routes, and responses.

pub mod models;
pub mod server;

pub mod responses;

pub use models::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestContext};
pub use server::HttpServer;

pub use responses::ResponseFormatter; 