//! HTTP infrastructure module
//! 
//! This module contains HTTP-related concerns including models,
//! server implementation, routes, utilities, responses, handlers, and processors.

pub mod models;
pub mod server;
pub mod utils;
pub mod responses;
pub mod handlers;
pub mod processors;
pub mod routes;
pub mod mining_pool;

pub use models::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestContext};
pub use server::HttpServer;
pub use utils::*;
pub use responses::ResponseFormatter;
pub use handlers::*;
pub use processors::{BaseRequestProcessor, RpcRequestProcessor}; 