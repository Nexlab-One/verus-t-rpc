use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_utility(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "help".to_string(),
        description: "Get help information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });
}


