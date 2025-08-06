//! Domain validation logic - Core validation business rules and models

use crate::shared::error::AppResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, value::RawValue};
use std::collections::HashMap;

/// RPC method definition with validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMethodDefinition {
    /// Method name
    pub name: String,
    
    /// Method description
    pub description: String,
    
    /// Whether this method is read-only
    pub read_only: bool,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
    
    /// Parameter validation rules
    pub parameter_rules: Vec<ParameterValidationRule>,
    
    /// Security level (low, medium, high)
    pub security_level: SecurityLevel,
    
    /// Whether method is enabled
    pub enabled: bool,
}

/// Parameter validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidationRule {
    /// Parameter index
    pub index: usize,
    
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub param_type: ParameterType,
    
    /// Whether parameter is required
    pub required: bool,
    
    /// Validation constraints
    pub constraints: Vec<ValidationConstraint>,
    
    /// Default value (if optional)
    pub default_value: Option<Value>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationConstraint {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Enum(Vec<String>),
    Custom(String),
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

/// Method registry for RPC validation
pub struct MethodRegistry {
    methods: HashMap<String, RpcMethodDefinition>,
}

impl MethodRegistry {
    /// Create a new method registry
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
        };
        
        // Register all supported methods
        registry.register_default_methods();
        
        registry
    }
    
    /// Register a method definition
    pub fn register_method(&mut self, method: RpcMethodDefinition) {
        self.methods.insert(method.name.clone(), method);
    }
    
    /// Get a method definition
    pub fn get_method(&self, name: &str) -> Option<&RpcMethodDefinition> {
        self.methods.get(name)
    }
    
    /// Check if a method is allowed
    pub fn is_method_allowed(&self, name: &str) -> bool {
        self.methods.get(name)
            .map(|method| method.enabled)
            .unwrap_or(false)
    }
    
    /// Validate method parameters
    pub fn validate_method_parameters(&self, method_name: &str, params: &[Box<RawValue>]) -> AppResult<()> {
        let method = self.methods.get(method_name)
            .ok_or_else(|| crate::shared::error::AppError::MethodNotAllowed {
                method: method_name.to_string(),
            })?;
        
        // Check parameter count
        if params.len() > method.parameter_rules.len() {
            return Err(crate::shared::error::AppError::InvalidParameters {
                method: method_name.to_string(),
                reason: format!("Too many parameters: expected {}, got {}", method.parameter_rules.len(), params.len()),
            });
        }
        
        // Validate each parameter
        for (i, rule) in method.parameter_rules.iter().enumerate() {
            if i < params.len() {
                self.validate_parameter(&params[i], rule)?;
            } else if rule.required {
                return Err(crate::shared::error::AppError::InvalidParameters {
                    method: method_name.to_string(),
                    reason: format!("Missing required parameter: {}", rule.name),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate a single parameter
    fn validate_parameter(&self, param: &RawValue, rule: &ParameterValidationRule) -> AppResult<()> {
        let value: Value = serde_json::from_str(&param.to_string())
            .map_err(|e| crate::shared::error::AppError::InvalidParameters {
                method: "unknown".to_string(),
                reason: format!("Invalid JSON in parameter {}: {}", rule.name, e),
            })?;
        
        // Check type
        if !self.matches_type(&value, &rule.param_type) {
            return Err(crate::shared::error::AppError::InvalidParameters {
                method: "unknown".to_string(),
                reason: format!("Parameter {} has wrong type", rule.name),
            });
        }
        
        // Apply constraints
        for constraint in &rule.constraints {
            self.apply_constraint(&value, constraint, &rule.name)?;
        }
        
        Ok(())
    }
    
    /// Check if value matches parameter type
    fn matches_type(&self, value: &Value, param_type: &ParameterType) -> bool {
        match param_type {
            ParameterType::String => matches!(value, Value::String(_)),
            ParameterType::Number => matches!(value, Value::Number(_)),
            ParameterType::Boolean => matches!(value, Value::Bool(_)),
            ParameterType::Object => matches!(value, Value::Object(_)),
            ParameterType::Array => matches!(value, Value::Array(_)),
            ParameterType::Any => true,
        }
    }
    
    /// Apply validation constraint
    fn apply_constraint(&self, value: &Value, constraint: &ValidationConstraint, param_name: &str) -> AppResult<()> {
        match constraint {
            ValidationConstraint::MinLength(min_len) => {
                if let Value::String(s) = value {
                    if s.len() < *min_len {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} too short: minimum length is {}", param_name, min_len),
                        });
                    }
                }
            },
            ValidationConstraint::MaxLength(max_len) => {
                if let Value::String(s) = value {
                    if s.len() > *max_len {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} too long: maximum length is {}", param_name, max_len),
                        });
                    }
                }
            },
            ValidationConstraint::MinValue(min_val) => {
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        if f < *min_val {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: "unknown".to_string(),
                                reason: format!("Parameter {} too small: minimum value is {}", param_name, min_val),
                            });
                        }
                    }
                }
            },
            ValidationConstraint::MaxValue(max_val) => {
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        if f > *max_val {
                            return Err(crate::shared::error::AppError::InvalidParameters {
                                method: "unknown".to_string(),
                                reason: format!("Parameter {} too large: maximum value is {}", param_name, max_val),
                            });
                        }
                    }
                }
            },
            ValidationConstraint::Pattern(pattern) => {
                if let Value::String(s) = value {
                    use regex::Regex;
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            if !regex.is_match(s) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} doesn't match pattern: {}", param_name, pattern),
                                });
                            }
                        }
                        Err(e) => {
                            return Err(crate::shared::error::AppError::Validation(
                                format!("Invalid regex pattern '{}': {}", pattern, e)
                            ));
                        }
                    }
                }
            },
            ValidationConstraint::Enum(allowed_values) => {
                if let Value::String(s) = value {
                    if !allowed_values.contains(s) {
                        return Err(crate::shared::error::AppError::InvalidParameters {
                            method: "unknown".to_string(),
                            reason: format!("Parameter {} must be one of: {:?}", param_name, allowed_values),
                        });
                    }
                }
            },
            ValidationConstraint::Custom(validation_name) => {
                // Custom validation functions
                match validation_name.as_str() {
                    "hex_string" => {
                        if let Value::String(s) = value {
                            if !s.chars().all(|c| c.is_ascii_hexdigit()) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid hex string", param_name),
                                });
                            }
                        }
                    },
                    "base58_string" => {
                        if let Value::String(s) = value {
                            if !s.chars().all(|c| c.is_alphanumeric() && !"0OIl".contains(c)) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid Base58 string", param_name),
                                });
                            }
                        }
                    },
                    "block_hash" => {
                        if let Value::String(s) = value {
                            if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
                                return Err(crate::shared::error::AppError::InvalidParameters {
                                    method: "unknown".to_string(),
                                    reason: format!("Parameter {} must be a valid 64-character hex block hash", param_name),
                                });
                            }
                        }
                    },
                    _ => {
                        return Err(crate::shared::error::AppError::Validation(
                            format!("Unknown custom validation: {}", validation_name)
                        ));
                    }
                }
            },
        }
        
        Ok(())
    }
    
    /// Register default methods
    fn register_default_methods(&mut self) {
        // Register all Verus RPC methods with comprehensive validation
        self.register_verus_methods();
    }

    /// Register all Verus RPC methods with proper validation rules
    fn register_verus_methods(&mut self) {
        // === READ-ONLY METHODS (Low Security) ===
        
        // Basic blockchain info methods
        self.register_method(RpcMethodDefinition {
            name: "getinfo".to_string(),
            description: "Get general information about the node".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getblockchaininfo".to_string(),
            description: "Get blockchain information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getblockcount".to_string(),
            description: "Get current block count".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getdifficulty".to_string(),
            description: "Get current difficulty".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getmempoolinfo".to_string(),
            description: "Get mempool information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getmininginfo".to_string(),
            description: "Get mining information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getnetworkinfo".to_string(),
            description: "Get network information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        // Block operations
        self.register_method(RpcMethodDefinition {
            name: "getblock".to_string(),
            description: "Get block information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "hash".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(64),
                        ValidationConstraint::MaxLength(64),
                    ],
                    default_value: None,
                },
                ParameterValidationRule {
                    index: 1,
                    name: "verbose".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    constraints: vec![],
                    default_value: Some(Value::Bool(true)),
                },
            ],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getblockhash".to_string(),
            description: "Get block hash by height".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "height".to_string(),
                    param_type: ParameterType::Number,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinValue(0.0),
                    ],
                    default_value: None,
                },
            ],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        self.register_method(RpcMethodDefinition {
            name: "getblockheader".to_string(),
            description: "Get block header".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "hash".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(64),
                        ValidationConstraint::MaxLength(64),
                    ],
                    default_value: None,
                },
            ],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        // Transaction operations
        self.register_method(RpcMethodDefinition {
            name: "getrawtransaction".to_string(),
            description: "Get raw transaction".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "txid".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(64),
                        ValidationConstraint::MaxLength(64),
                    ],
                    default_value: None,
                },
                ParameterValidationRule {
                    index: 1,
                    name: "verbose".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![
                        ValidationConstraint::MinValue(0.0),
                        ValidationConstraint::MaxValue(1.0),
                    ],
                    default_value: Some(Value::Number(serde_json::Number::from(0))),
                },
            ],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        // === WRITE METHODS (High Security) ===

        self.register_method(RpcMethodDefinition {
            name: "sendrawtransaction".to_string(),
            description: "Send a raw transaction".to_string(),
            read_only: false,
            required_permissions: vec!["send_transaction".to_string()],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "hexstring".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(1),
                        ValidationConstraint::MaxLength(100000),
                    ],
                    default_value: None,
                },
            ],
            security_level: SecurityLevel::High,
            enabled: true,
        });

        // === IDENTITY METHODS (Medium Security) ===

        self.register_method(RpcMethodDefinition {
            name: "getidentity".to_string(),
            description: "Get identity information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "identity".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(1),
                        ValidationConstraint::MaxLength(100),
                    ],
                    default_value: None,
                },
                ParameterValidationRule {
                    index: 1,
                    name: "height".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![
                        ValidationConstraint::MinValue(0.0),
                    ],
                    default_value: Some(Value::Number(serde_json::Number::from(-1))),
                },
                ParameterValidationRule {
                    index: 2,
                    name: "txproof".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    constraints: vec![],
                    default_value: Some(Value::Bool(false)),
                },
                ParameterValidationRule {
                    index: 3,
                    name: "txproofheight".to_string(),
                    param_type: ParameterType::Number,
                    required: false,
                    constraints: vec![
                        ValidationConstraint::MinValue(0.0),
                    ],
                    default_value: Some(Value::Number(serde_json::Number::from(-1))),
                },
            ],
            security_level: SecurityLevel::Medium,
            enabled: true,
        });

        // === CURRENCY METHODS (Medium Security) ===

        self.register_method(RpcMethodDefinition {
            name: "getcurrency".to_string(),
            description: "Get currency information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![
                ParameterValidationRule {
                    index: 0,
                    name: "currency".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    constraints: vec![
                        ValidationConstraint::MinLength(1),
                        ValidationConstraint::MaxLength(100),
                    ],
                    default_value: None,
                },
            ],
            security_level: SecurityLevel::Medium,
            enabled: true,
        });

        // === UTILITY METHODS (Low Security) ===

        self.register_method(RpcMethodDefinition {
            name: "help".to_string(),
            description: "Get help information".to_string(),
            read_only: true,
            required_permissions: vec![],
            parameter_rules: vec![],
            security_level: SecurityLevel::Low,
            enabled: true,
        });

        // Register additional methods with basic validation
        self.register_additional_methods();
    }

    /// Register additional methods that don't need complex validation
    fn register_additional_methods(&mut self) {
        let additional_methods = vec![
            // Block operations
            ("getbestblockhash", "Get best block hash", true, vec![], vec![]),
            ("getblockhashes", "Get block hashes", true, vec![], vec![
                ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("count", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
            ]),
            ("getblocksubsidy", "Get block subsidy", true, vec![], vec![
                ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("getblocktemplate", "Get block template", true, vec![], vec![
                ("template_request", ParameterType::Object, true, vec![]),
            ]),
            ("getchaintips", "Get chain tips", true, vec![], vec![]),
            
            // Address operations
            ("getaddressbalance", "Get address balance", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getaddressutxos", "Get address UTXOs", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getaddressdeltas", "Get address deltas", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getaddresstxids", "Get address transaction IDs", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getaddressmempool", "Get address mempool", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            
            // Transaction operations
            ("getrawmempool", "Get raw mempool", true, vec![], vec![]),
            ("gettxout", "Get transaction output", true, vec![], vec![
                ("txid", ParameterType::String, true, vec![ValidationConstraint::MinLength(64)]),
                ("n", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("include_mempool", ParameterType::Boolean, false, vec![]),
            ]),
            ("gettxoutsetinfo", "Get transaction output set info", true, vec![], vec![]),
            ("getspentinfo", "Get spent info", true, vec![], vec![
                ("txid", ParameterType::Object, true, vec![]),
            ]),
            
            // Currency operations
            ("getcurrencystate", "Get currency state", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getcurrencyconverters", "Get currency converters", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getcurrencytrust", "Get currency trust", true, vec![], vec![
                ("addresses", ParameterType::Array, true, vec![]),
            ]),
            ("getinitialcurrencystate", "Get initial currency state", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            
            // Identity operations
            ("getidentitieswithaddress", "Get identities with address", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getidentitieswithrevocation", "Get identities with revocation", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getidentitieswithrecovery", "Get identities with recovery", true, vec![], vec![
                ("addresses", ParameterType::Object, true, vec![]),
            ]),
            ("getidentitytrust", "Get identity trust", true, vec![], vec![
                ("addresses", ParameterType::Array, true, vec![]),
            ]),
            ("getidentitycontent", "Get identity content", true, vec![], vec![
                ("identity", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("txproofheight", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("txproof", ParameterType::Boolean, false, vec![]),
                ("txproofheight", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
                ("content", ParameterType::String, false, vec![]),
                ("contentproof", ParameterType::Boolean, false, vec![]),
            ]),
            
            // Utility operations
            ("createmultisig", "Create multi-signature", true, vec![], vec![
                ("nrequired", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
                ("keys", ParameterType::Array, true, vec![]),
            ]),
            ("createrawtransaction", "Create raw transaction", true, vec![], vec![
                ("inputs", ParameterType::Array, true, vec![]),
                ("outputs", ParameterType::Object, true, vec![]),
                ("locktime", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
                ("expiryheight", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("decoderawtransaction", "Decode raw transaction", true, vec![], vec![
                ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("iswitness", ParameterType::Boolean, false, vec![]),
            ]),
            ("decodescript", "Decode script", true, vec![], vec![
                ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("iswitness", ParameterType::Boolean, false, vec![]),
            ]),
            ("estimatefee", "Estimate fee", true, vec![], vec![
                ("nblocks", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
            ]),
            ("estimatepriority", "Estimate priority", true, vec![], vec![
                ("nblocks", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
            ]),
            ("verifymessage", "Verify message", true, vec![], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("signature", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("message", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("checkexpiry", ParameterType::Boolean, false, vec![]),
            ]),
            ("verifyhash", "Verify hash", true, vec![], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("signature", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("hash", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("checkexpiry", ParameterType::Boolean, false, vec![]),
            ]),
            ("verifysignature", "Verify signature", true, vec![], vec![
                ("signature", ParameterType::Object, true, vec![]),
            ]),
            ("hashdata", "Hash data", true, vec![], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("hexstring", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("messagetype", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("convertpassphrase", "Convert passphrase", true, vec![], vec![
                ("passphrase", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getvdxfid", "Get VDXF ID", true, vec![], vec![
                ("vdxfkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("vdxfobj", ParameterType::Object, false, vec![]),
            ]),
            ("getlastimportfrom", "Get last import from", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getlaunchinfo", "Get launch info", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getpendingtransfers", "Get pending transfers", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getreservedeposits", "Get reserved deposits", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getsaplingtree", "Get Sapling tree", true, vec![], vec![
                ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("getexports", "Get exports", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("height", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("count", ParameterType::Number, true, vec![ValidationConstraint::MinValue(1.0)]),
            ]),
            ("getnotarizationdata", "Get notarization data", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("getoffers", "Get offers", true, vec![], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("fromcurrency", ParameterType::Boolean, false, vec![]),
                ("tocurrency", ParameterType::Boolean, false, vec![]),
            ]),
            ("makeOffer", "Create marketplace offer", false, vec!["write".to_string()], vec![
                ("currency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("offer", ParameterType::Object, true, vec![]),
                ("fromcurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("tocurrency", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("amount", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("price", ParameterType::Number, true, vec![ValidationConstraint::MinValue(0.0)]),
                ("expiry", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("z_getnewaddress", "Get new Z-address", true, vec![], vec![
                ("type", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["sprout".to_string(), "sapling".to_string(), "orchard".to_string()])]),
            ]),
            ("z_listaddresses", "List Z-addresses", true, vec![], vec![]),
            ("z_getbalance", "Get Z-address balance", true, vec![], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("minconf", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("z_sendmany", "Send to multiple Z-addresses", false, vec!["write".to_string()], vec![
                ("fromaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("amounts", ParameterType::Array, true, vec![]),
                ("minconf", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
                ("fee", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("z_shieldcoinbase", "Shield coinbase funds to Z-address", false, vec!["write".to_string()], vec![
                ("fromaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("toaddress", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("fee", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
                ("limit", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
            ]),
            ("z_validateaddress", "Validate Z-address", true, vec![], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("z_viewtransaction", "View Z-transaction details", true, vec![], vec![
                ("txid", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("z_exportkey", "Export Z-address private key", false, vec!["write".to_string()], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("z_importkey", "Import Z-address private key", false, vec!["write".to_string()], vec![
                ("zkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("rescan", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["yes".to_string(), "no".to_string(), "whenkeyisnew".to_string()])]),
            ]),
            ("z_exportviewingkey", "Export Z-address viewing key", false, vec!["write".to_string()], vec![
                ("address", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
            ]),
            ("z_importviewingkey", "Import Z-address viewing key", false, vec!["write".to_string()], vec![
                ("vkey", ParameterType::String, true, vec![ValidationConstraint::MinLength(1)]),
                ("rescan", ParameterType::String, false, vec![ValidationConstraint::Enum(vec!["yes".to_string(), "no".to_string(), "whenkeyisnew".to_string()])]),
            ]),
            ("listcurrencies", "List currencies", true, vec![], vec![
                ("currency", ParameterType::Object, false, vec![]),
                ("start", ParameterType::Number, false, vec![ValidationConstraint::MinValue(0.0)]),
                ("count", ParameterType::Number, false, vec![ValidationConstraint::MinValue(1.0)]),
            ]),
            ("coinsupply", "Get coin supply", true, vec![], vec![]),
            ("getbestproofroot", "Get best proof root", true, vec![], vec![
                ("proofroot", ParameterType::Object, true, vec![]),
            ]),
        ];

        for (name, description, read_only, permissions, param_rules) in additional_methods {
            let mut parameter_rules = Vec::new();
            for (i, (param_name, param_type, required, constraints)) in param_rules.iter().enumerate() {
                parameter_rules.push(ParameterValidationRule {
                    index: i,
                    name: param_name.to_string(),
                    param_type: param_type.clone(),
                    required: *required,
                    constraints: constraints.clone(),
                    default_value: None,
                });
            }

            self.register_method(RpcMethodDefinition {
                name: name.to_string(),
                description: description.to_string(),
                read_only,
                required_permissions: permissions,
                parameter_rules,
                security_level: if read_only { SecurityLevel::Low } else { SecurityLevel::Medium },
                enabled: true,
            });
        }
    }
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain validator that uses the method registry
pub struct DomainValidator {
    registry: MethodRegistry,
}

impl DomainValidator {
    /// Create a new domain validator
    pub fn new() -> Self {
        Self {
            registry: MethodRegistry::new(),
        }
    }
    
    /// Validate a method call
    pub fn validate_method_call(&self, method: &str, params: &Option<Value>) -> AppResult<()> {
        // Check if method is allowed
        if !self.registry.is_method_allowed(method) {
            return Err(crate::shared::error::AppError::MethodNotAllowed {
                method: method.to_string(),
            });
        }
        
        // Convert params to raw format for validation
        let raw_params = if let Some(params) = params {
            if let Some(array) = params.as_array() {
                let raw_params: Vec<Box<RawValue>> = array
                    .iter()
                    .map(|v| RawValue::from_string(v.to_string())
                        .map_err(|e| crate::shared::error::AppError::Internal(format!("Failed to create raw value: {}", e))))
                    .collect::<AppResult<Vec<Box<RawValue>>>>()?;
                raw_params
            } else {
                return Err(crate::shared::error::AppError::InvalidParameters {
                    method: method.to_string(),
                    reason: "Parameters must be an array".to_string(),
                });
            }
        } else {
            vec![]
        };
        
        // Validate parameters
        self.registry.validate_method_parameters(method, &raw_params)?;
        
        Ok(())
    }
    
    /// Get method definition
    pub fn get_method_definition(&self, method: &str) -> Option<&RpcMethodDefinition> {
        self.registry.get_method(method)
    }
    
    /// Check if method is read-only
    pub fn is_method_read_only(&self, method: &str) -> bool {
        self.registry.get_method(method)
            .map(|m| m.read_only)
            .unwrap_or(true) // Default to read-only for safety
    }
    
    /// Get required permissions for a method
    pub fn get_required_permissions(&self, method: &str) -> Vec<String> {
        self.registry.get_method(method)
            .map(|m| m.required_permissions.clone())
            .unwrap_or_default()
    }
}

impl Default for DomainValidator {
    fn default() -> Self {
        Self::new()
    }
} 