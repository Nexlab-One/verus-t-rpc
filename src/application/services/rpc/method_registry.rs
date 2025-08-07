//! Method registry and lookup for RPC methods

use crate::domain::rpc::{Constraint, ParameterRule, ParameterType, RpcMethod};

pub fn get_method_info(method_name: &str) -> Option<RpcMethod> {
    let method_registry = [
        ("getinfo", RpcMethod {
            name: "getinfo".to_string(),
            description: "Get blockchain information".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![],
        }),
        ("getblock", RpcMethod {
            name: "getblock".to_string(),
            description: "Get block information".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "hash".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(64)],
                }
            ],
        }),
        ("sendrawtransaction", RpcMethod {
            name: "sendrawtransaction".to_string(),
            description: "Send raw transaction".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "hex".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(100)],
                }
            ],
        }),
        ("makeOffer", RpcMethod {
            name: "makeOffer".to_string(),
            description: "Create marketplace offer".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "currency".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "offer".to_string(),
                    param_type: ParameterType::Object,
                    required: true,
                    constraints: vec![],
                },
                ParameterRule {
                    index: 2,
                    name: "fromcurrency".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 3,
                    name: "tocurrency".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 4,
                    name: "amount".to_string(),
                    param_type: ParameterType::Number,
                    required: true,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
                ParameterRule {
                    index: 5,
                    name: "price".to_string(),
                    param_type: ParameterType::Number,
                    required: true,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
                ParameterRule {
                    index: 6,
                    name: "expiry".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
            ],
        }),
        ("z_getnewaddress", RpcMethod {
            name: "z_getnewaddress".to_string(),
            description: "Get new Z-address".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "type".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    constraints: vec![Constraint::Custom("sprout|sapling|orchard".to_string())],
                },
            ],
        }),
        ("z_listaddresses", RpcMethod {
            name: "z_listaddresses".to_string(),
            description: "List Z-addresses".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![],
        }),
        ("z_getbalance", RpcMethod {
            name: "z_getbalance".to_string(),
            description: "Get Z-address balance".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "address".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "minconf".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
            ],
        }),
        ("z_sendmany", RpcMethod {
            name: "z_sendmany".to_string(),
            description: "Send to multiple Z-addresses".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "fromaddress".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "amounts".to_string(),
                    param_type: ParameterType::Array,
                    required: true,
                    constraints: vec![],
                },
                ParameterRule {
                    index: 2,
                    name: "minconf".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
                ParameterRule {
                    index: 3,
                    name: "fee".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
            ],
        }),
        ("z_shieldcoinbase", RpcMethod {
            name: "z_shieldcoinbase".to_string(),
            description: "Shield coinbase funds to Z-address".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "fromaddress".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "toaddress".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 2,
                    name: "fee".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
                ParameterRule {
                    index: 3,
                    name: "limit".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![Constraint::MinValue(0.0)],
                },
            ],
        }),
        ("z_validateaddress", RpcMethod {
            name: "z_validateaddress".to_string(),
            description: "Validate Z-address".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "address".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
            ],
        }),
        ("z_viewtransaction", RpcMethod {
            name: "z_viewtransaction".to_string(),
            description: "View Z-transaction details".to_string(),
            read_only: true,
            required_permissions: vec!["read".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "txid".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
            ],
        }),
        ("z_exportkey", RpcMethod {
            name: "z_exportkey".to_string(),
            description: "Export Z-address private key".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "address".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
            ],
        }),
        ("z_importkey", RpcMethod {
            name: "z_importkey".to_string(),
            description: "Import Z-address private key".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "zkey".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "rescan".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    constraints: vec![Constraint::Custom("yes|no|whenkeyisnew".to_string())],
                },
            ],
        }),
        ("z_exportviewingkey", RpcMethod {
            name: "z_exportviewingkey".to_string(),
            description: "Export Z-address viewing key".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "address".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
            ],
        }),
        ("z_importviewingkey", RpcMethod {
            name: "z_importviewingkey".to_string(),
            description: "Import Z-address viewing key".to_string(),
            read_only: false,
            required_permissions: vec!["write".to_string()],
            parameter_rules: vec![
                ParameterRule {
                    index: 0,
                    name: "vkey".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![Constraint::MinLength(1)],
                },
                ParameterRule {
                    index: 1,
                    name: "rescan".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    constraints: vec![Constraint::Custom("yes|no|whenkeyisnew".to_string())],
                },
            ],
        }),
    ];

    method_registry
        .iter()
        .find(|(name, _)| *name == method_name)
        .map(|(_, method)| method.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_existing_method() {
        let m = get_method_info("getinfo");
        assert!(m.is_some());
        let method = m.unwrap();
        assert_eq!(method.name, "getinfo");
        assert!(method.read_only);
    }

    #[test]
    fn get_unknown_method() {
        assert!(get_method_info("does_not_exist").is_none());
    }

    #[test]
    fn getblock_rules_include_hash_minlen() {
        let m = get_method_info("getblock").expect("method exists");
        let hash_rule = m.parameter_rules.iter().find(|r| r.name == "hash").expect("hash rule");
        assert!(matches!(hash_rule.constraints.get(0), Some(Constraint::MinLength(64))));
    }
}


