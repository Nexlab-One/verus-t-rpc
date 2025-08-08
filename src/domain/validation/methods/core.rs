use crate::domain::validation::types::{RpcMethodDefinition, SecurityLevel};
use crate::domain::validation::registry::MethodRegistry;

pub fn register_core(registry: &mut MethodRegistry) {
    registry.register_method(RpcMethodDefinition {
        name: "getinfo".to_string(),
        description: "Get general information about the node".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getblockchaininfo".to_string(),
        description: "Get blockchain information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getblockcount".to_string(),
        description: "Get current block count".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getdifficulty".to_string(),
        description: "Get current difficulty".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getmempoolinfo".to_string(),
        description: "Get mempool information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getmininginfo".to_string(),
        description: "Get mining information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });

    registry.register_method(RpcMethodDefinition {
        name: "getnetworkinfo".to_string(),
        description: "Get network information".to_string(),
        read_only: true,
        required_permissions: vec![],
        parameter_rules: vec![],
        security_level: SecurityLevel::Low,
        enabled: true,
    });
}


