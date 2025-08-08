//! Domain layer - Core business logic and domain models
//! 
//! This module contains the core business logic, domain models, and business rules
//! that are independent of infrastructure concerns like HTTP, databases, etc.

pub mod rpc;
pub mod security;
pub mod validation;
pub mod payments;

// Re-export specific types to avoid conflicts
pub use rpc::{
    RpcRequest, RpcResponse, RpcError, ClientInfo, ErrorContext,
    RpcMethod, ParameterRule, Constraint,
};
pub use security::{
    SecurityPolicy, SecurityValidator, SecurityContext,
    MethodSecurityRule, GlobalSecuritySettings, RateLimitSettings, ValidationRule,
};
pub use validation::{
    DomainValidator, MethodRegistry, RpcMethodDefinition,
    ParameterValidationRule, ValidationConstraint,
}; 
pub use payments::{PaymentSession, PaymentStatus, PaymentTier, ShieldedAddressType};