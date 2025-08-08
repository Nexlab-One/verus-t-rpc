/// Comprehensive method validator for RPC requests
/// 
/// This validator provides detailed parameter validation for all Verus RPC methods,
/// ensuring type safety and parameter constraints are enforced before requests
/// are forwarded to the daemon.

use crate::shared::error::{AppError, AppResult};
use serde_json::{Value, value::RawValue};
use std::collections::HashMap;

pub struct ComprehensiveValidator {
    /// Cache for compiled validation rules
    validation_cache: HashMap<String, ValidationRule>,
}

/// Validation rule for a method
#[derive(Debug, Clone)]
struct ValidationRule {
    /// Expected parameter types
    expected_types: Vec<ParameterType>,
    /// Custom validation function
    custom_validator: Option<fn(&[Box<RawValue>]) -> bool>,
}

/// Parameter types for validation
#[derive(Debug, Clone)]
enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Float,
    Integer,
}

impl ComprehensiveValidator {
    /// Create a new validator
    pub fn new() -> Self {
        let mut validator = Self {
            validation_cache: HashMap::new(),
        };
        
        // Initialize validation rules for all supported methods
        validator.initialize_validation_rules();
        
        validator
    }

    /// Validate a method and its parameters
    pub fn validate_method(&self, method: &str, params: &Option<Value>) -> AppResult<()> {
        // Convert params to the format expected by the validation logic
        let raw_params = if let Some(params) = params {
            if let Some(array) = params.as_array() {
                let raw_params: Vec<Box<RawValue>> = array
                    .iter()
                    .map(|v| RawValue::from_string(v.to_string())
                        .map_err(|e| AppError::Internal(format!("Failed to create raw value: {}", e))))
                    .collect::<AppResult<Vec<Box<RawValue>>>>()?;
                Some(raw_params)
            } else {
                return Err(AppError::InvalidParameters {
                    method: method.to_string(),
                    reason: "Parameters must be an array".to_string(),
                });
            }
        } else {
            None
        };

        let params_slice = raw_params.as_deref().unwrap_or(&[]);
        
        if !self.is_method_allowed(method, params_slice) {
            return Err(AppError::MethodNotAllowed {
                method: method.to_string(),
            });
        }

        Ok(())
    }

    /// Check if a method is allowed with the given parameters
    fn is_method_allowed(&self, method: &str, params: &[Box<RawValue>]) -> bool {
        // First check if we have a cached validation rule
        if let Some(rule) = self.validation_cache.get(method) {
            // Use cached rule for validation
            if let Some(custom_validator) = rule.custom_validator {
                return custom_validator(params);
            } else {
                return self.check_params(params, &rule.expected_types);
            }
        }

        // Fall back to direct validation for methods not in cache
        match method {
            // === COMPLEX VALIDATION METHODS ===
            "fundrawtransaction" => self.validate_fundrawtransaction(params),
            "signdata" => self.validate_signdata(params),
            "recoveridentity" => self.validate_recoveridentity(params),
            "registeridentity" => self.validate_registeridentity(params),
            "revokeidentity" => self.validate_revokeidentity(params),
            "updateidentity" => self.validate_updateidentity(params),
            "setidentitytimelock" => self.validate_setidentitytimelock(params),
            "sendcurrency" => self.validate_sendcurrency(params),
            
            // === SIMPLE VALIDATION METHODS ===
            "coinsupply" => self.check_params(params, &[]),
            "convertpassphrase" => self.check_params(params, &[ParameterType::String]),
            "createmultisig" => self.check_params(params, &[ParameterType::Integer, ParameterType::Array]),
            "createrawtransaction" => self.check_params(params, &[ParameterType::Array, ParameterType::Object, ParameterType::Integer, ParameterType::Integer]),
            "decoderawtransaction" => self.check_params(params, &[ParameterType::String, ParameterType::Boolean]),
            "decodescript" => self.check_params(params, &[ParameterType::String, ParameterType::Boolean]),
            "estimateconversion" => self.check_params(params, &[ParameterType::Object]),
            "estimatefee" => self.check_params(params, &[ParameterType::Integer]),
            "estimatepriority" => self.check_params(params, &[ParameterType::Integer]),
            "getaddressmempool" => self.check_params(params, &[ParameterType::Object]),
            "getaddressutxos" => self.check_params(params, &[ParameterType::Object]),
            "getaddressbalance" => self.check_params(params, &[ParameterType::Object]),
            "getaddressdeltas" => self.check_params(params, &[ParameterType::Object]),
            "getaddresstxids" => self.check_params(params, &[ParameterType::Object]),
            "getbestblockhash" => self.check_params(params, &[]),
            "getbestproofroot" => self.check_params(params, &[ParameterType::Object]),
            "getblock" => self.check_params(params, &[ParameterType::String, ParameterType::Boolean]),
            "getblockchaininfo" => self.check_params(params, &[]),
            "getblockcount" => self.check_params(params, &[]),
            "getblockhashes" => self.check_params(params, &[ParameterType::Integer, ParameterType::Integer]),
            "getblockhash" => self.check_params(params, &[ParameterType::Integer]),
            "getblockheader" => self.check_params(params, &[ParameterType::String]),
            "getblocksubsidy" => self.check_params(params, &[ParameterType::Integer]),
            "getblocktemplate" => self.check_params(params, &[ParameterType::Object]),
            "getchaintips" => self.check_params(params, &[]),
            "getcurrency" => self.check_params(params, &[ParameterType::String]),
            "getcurrencyconverters" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::String]),
            "getcurrencystate" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::String]),
            "getcurrencytrust" => self.check_params(params, &[ParameterType::Array]),
            "getdifficulty" => self.check_params(params, &[]),
            "getexports" => self.check_params(params, &[ParameterType::String, ParameterType::Integer, ParameterType::Integer]),
            "getinfo" => self.check_params(params, &[]),
            "getinitialcurrencystate" => self.check_params(params, &[ParameterType::String]),
            "getidentitieswithaddress" => self.check_params(params, &[ParameterType::Object]),
            "getidentitieswithrevocation" => self.check_params(params, &[ParameterType::Object]),
            "getidentitieswithrecovery" => self.check_params(params, &[ParameterType::Object]),
            "getidentity" => self.check_params(params, &[ParameterType::String, ParameterType::Integer, ParameterType::Boolean, ParameterType::Integer]),
            "getidentitytrust" => self.check_params(params, &[ParameterType::Array]),
            "getidentitycontent" => self.check_params(params, &[ParameterType::String, ParameterType::Integer, ParameterType::Integer, ParameterType::Boolean, ParameterType::Integer, ParameterType::String, ParameterType::Boolean]),
            "getlastimportfrom" => self.check_params(params, &[ParameterType::String]),
            "getlaunchinfo" => self.check_params(params, &[ParameterType::String]),
            "getmempoolinfo" => self.check_params(params, &[]),
            "getmininginfo" => self.check_params(params, &[]),
            "getnetworkinfo" => self.check_params(params, &[]),
            "getnotarizationdata" => self.check_params(params, &[ParameterType::String]),
            "getoffers" => self.check_params(params, &[ParameterType::String, ParameterType::Boolean, ParameterType::Boolean]),
            "makeOffer" => self.validate_make_offer(params),
            "getpendingtransfers" => self.check_params(params, &[ParameterType::String]),
            "getrawmempool" => self.check_params(params, &[]),
            "getrawtransaction" => self.check_params(params, &[ParameterType::String, ParameterType::Integer]),
            "getreservedeposits" => self.check_params(params, &[ParameterType::String]),
            "getsaplingtree" => self.check_params(params, &[ParameterType::Integer]),
            "getspentinfo" => self.check_params(params, &[ParameterType::Object]),
            "gettxout" => self.check_params(params, &[ParameterType::String, ParameterType::Integer, ParameterType::Boolean]),
            "gettxoutsetinfo" => self.check_params(params, &[]),
            "getvdxfid" => self.check_params(params, &[ParameterType::String, ParameterType::Object]),
            "hashdata" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::String]),
            "help" => self.check_params(params, &[]),
            "listcurrencies" => self.check_params(params, &[ParameterType::Object, ParameterType::Integer, ParameterType::Integer]),
            "sendrawtransaction" => self.check_params(params, &[ParameterType::String]),
            "submitacceptednotarization" => self.check_params(params, &[ParameterType::Object, ParameterType::Object]),
            "submitimports" => self.check_params(params, &[ParameterType::Object]),
            "verifymessage" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::String, ParameterType::Boolean]),
            "verifyhash" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::String, ParameterType::Boolean]),
            "verifysignature" => self.check_params(params, &[ParameterType::Object]),
            // Z-cash operations
            "z_getnewaddress" => self.check_params(params, &[ParameterType::String]),
            "z_listaddresses" => self.check_params(params, &[]),
            "z_getbalance" => self.check_params(params, &[ParameterType::String, ParameterType::Integer]),
            "z_sendmany" => self.validate_z_sendmany(params),
            "z_shieldcoinbase" => self.check_params(params, &[ParameterType::String, ParameterType::String, ParameterType::Float, ParameterType::Integer]),
            "z_validateaddress" => self.check_params(params, &[ParameterType::String]),
            "z_viewtransaction" => self.check_params(params, &[ParameterType::String]),
            "z_exportkey" => self.check_params(params, &[ParameterType::String]),
            "z_importkey" => self.check_params(params, &[ParameterType::String, ParameterType::String]),
            "z_exportviewingkey" => self.check_params(params, &[ParameterType::String]),
            "z_importviewingkey" => self.check_params(params, &[ParameterType::String, ParameterType::String]),
            _ => false,
        }
    }

    /// Initialize validation rules for all supported methods
    fn initialize_validation_rules(&mut self) {
        // Cache validation rules for performance optimization
        self.validation_cache.insert("getinfo".to_string(), ValidationRule {
            expected_types: vec![],
            custom_validator: None,
        });

        self.validation_cache.insert("getblock".to_string(), ValidationRule {
            expected_types: vec![ParameterType::String, ParameterType::Boolean],
            custom_validator: None,
        });

        self.validation_cache.insert("getblockcount".to_string(), ValidationRule {
            expected_types: vec![],
            custom_validator: None,
        });

        self.validation_cache.insert("getdifficulty".to_string(), ValidationRule {
            expected_types: vec![],
            custom_validator: None,
        });

        self.validation_cache.insert("getrawtransaction".to_string(), ValidationRule {
            expected_types: vec![ParameterType::String, ParameterType::Number], // Using Number variant
            custom_validator: None,
        });

        self.validation_cache.insert("sendrawtransaction".to_string(), ValidationRule {
            expected_types: vec![ParameterType::String],
            custom_validator: None,
        });

        // Cache complex validation methods with custom validators
        self.validation_cache.insert("fundrawtransaction".to_string(), ValidationRule {
            expected_types: vec![ParameterType::String, ParameterType::Array, ParameterType::String, ParameterType::Number],
            custom_validator: Some(|params| {
                if params.len() != 4 {
                    return false;
                }
                
                let (param1, param2, param3, param4) = (
                    serde_json::from_str::<Value>(&params[0].to_string()),
                    serde_json::from_str::<Value>(&params[1].to_string()),
                    serde_json::from_str::<Value>(&params[2].to_string()),
                    serde_json::from_str::<Value>(&params[3].to_string()),
                );

                matches!(
                    (param1, param2, param3, param4),
                    (Ok(Value::String(_)), Ok(Value::Array(_)), Ok(Value::String(_)), Ok(Value::Number(_)))
                )
            }),
        });

        self.validation_cache.insert("signdata".to_string(), ValidationRule {
            expected_types: vec![ParameterType::Object],
            custom_validator: Some(|params| {
                if params.len() != 1 {
                    return false;
                }
                
                if let Ok(value) = serde_json::from_str::<Value>(&params[0].to_string()) {
                    if let Value::Object(obj) = value {
                        !obj.contains_key("address")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }),
        });
    }

    /// Validate fundrawtransaction method
    fn validate_fundrawtransaction(&self, params: &[Box<RawValue>]) -> bool {
        if params.len() != 4 {
            return false;
        }
        
        let (param1, param2, param3, param4) = (
            serde_json::from_str::<Value>(&params[0].to_string()),
            serde_json::from_str::<Value>(&params[1].to_string()),
            serde_json::from_str::<Value>(&params[2].to_string()),
            serde_json::from_str::<Value>(&params[3].to_string()),
        );

        matches!(
            (param1, param2, param3, param4),
            (Ok(Value::String(_)), Ok(Value::Array(_)), Ok(Value::String(_)), Ok(Value::Number(_)))
        )
    }

    /// Validate signdata method
    fn validate_signdata(&self, params: &[Box<RawValue>]) -> bool {
        if params.len() != 1 {
            return false;
        }
        
        if let Ok(value) = serde_json::from_str::<Value>(&params[0].to_string()) {
            if let Value::Object(obj) = value {
                !obj.contains_key("address")
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Validate recoveridentity method
    fn validate_recoveridentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::Object, ParameterType::Boolean, ParameterType::Boolean, ParameterType::Float, ParameterType::String])
    }

    /// Validate registeridentity method
    fn validate_registeridentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::Object, ParameterType::Boolean, ParameterType::Float, ParameterType::String])
    }

    /// Validate revokeidentity method
    fn validate_revokeidentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::String, ParameterType::Boolean, ParameterType::Boolean, ParameterType::Float, ParameterType::String])
    }

    /// Validate updateidentity method
    fn validate_updateidentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::Object, ParameterType::Boolean, ParameterType::Boolean, ParameterType::Float, ParameterType::String])
    }

    /// Validate setidentitytimelock method
    fn validate_setidentitytimelock(&self, params: &[Box<RawValue>]) -> bool {
        params.get(2)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::String, ParameterType::Object, ParameterType::Boolean, ParameterType::Float, ParameterType::String])
    }

    /// Validate sendcurrency method
    fn validate_sendcurrency(&self, params: &[Box<RawValue>]) -> bool {
        params.get(4)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &[ParameterType::String, ParameterType::Array, ParameterType::Integer, ParameterType::Float, ParameterType::Boolean])
    }

    /// Validate makeOffer method for marketplace offer creation
    fn validate_make_offer(&self, params: &[Box<RawValue>]) -> bool {
        // Validate required parameters: currency, offer, fromcurrency, tocurrency, amount, price
        if params.len() < 6 {
            return false;
        }
        
        // Check that currency is a string
        if !self.check_params(&params[0..1], &[ParameterType::String]) {
            return false;
        }
        
        // Check that offer is an object
        if !self.check_params(&params[1..2], &[ParameterType::Object]) {
            return false;
        }
        
        // Check that fromcurrency and tocurrency are strings
        if !self.check_params(&params[2..4], &[ParameterType::String, ParameterType::String]) {
            return false;
        }
        
        // Check that amount and price are numbers
        if !self.check_params(&params[4..6], &[ParameterType::Number, ParameterType::Number]) {
            return false;
        }
        
        // Check optional expiry parameter if present
        if params.len() > 6 {
            if !self.check_params(&params[6..7], &[ParameterType::Number]) {
                return false;
            }
        }
        
        true
    }

    /// Validate z_sendmany method for sending to multiple Z-addresses
    fn validate_z_sendmany(&self, params: &[Box<RawValue>]) -> bool {
        // Validate required parameters: fromaddress, amounts
        if params.len() < 2 {
            return false;
        }
        
        // Check that fromaddress is a string
        if !self.check_params(&params[0..1], &[ParameterType::String]) {
            return false;
        }
        
        // Check that amounts is an array
        if !self.check_params(&params[1..2], &[ParameterType::Array]) {
            return false;
        }
        
        // Check optional minconf and fee parameters if present
        if params.len() > 2 {
            if !self.check_params(&params[2..3], &[ParameterType::Integer]) {
                return false;
            }
        }
        
        if params.len() > 3 {
            if !self.check_params(&params[3..4], &[ParameterType::Float]) {
                return false;
            }
        }
        
        true
    }

    /// Check parameter types against expected types
    fn check_params(&self, params: &[Box<RawValue>], expected_types: &[ParameterType]) -> bool {
        if params.len() > expected_types.len() {
            return false;
        }
        
        for (param, expected_type) in params.iter().zip(expected_types) {
            let value: Value = match serde_json::from_str(&param.to_string()) {
                Ok(v) => v,
                Err(_) => return false,
            };
            
            match expected_type {
                ParameterType::Object => if !matches!(value, Value::Object(_)) { return false; },
                ParameterType::Array => if !matches!(value, Value::Array(_)) { return false; },
                ParameterType::Integer => if !matches!(value, Value::Number(n) if n.is_i64()) { return false; },
                ParameterType::Float => if !matches!(value, Value::Number(n) if n.is_f64()) { return false; },
                ParameterType::String => if !matches!(value, Value::String(_)) { return false; },
                ParameterType::Boolean => if !matches!(value, Value::Bool(_)) { return false; },
                ParameterType::Number => if !matches!(value, Value::Number(_)) { return false; },
            }
        }
        
        true
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, Vec<String>) {
        let cache_size = self.validation_cache.len();
        let cached_methods: Vec<String> = self.validation_cache.keys().cloned().collect();
        (cache_size, cached_methods)
    }

    /// Clear validation cache (useful for testing or cache invalidation)
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }
}

impl Default for ComprehensiveValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getinfo_allowed() {
        let validator = ComprehensiveValidator::new();
        let params: Option<Value> = None;
        assert!(validator.validate_method("getinfo", &params).is_ok());
    }

    #[test]
    fn test_getblock_allowed() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            Value::Bool(true),
        ]));
        assert!(validator.validate_method("getblock", &params).is_ok());
    }

    #[test]
    fn test_invalid_method_not_allowed() {
        let validator = ComprehensiveValidator::new();
        let params: Option<Value> = None;
        assert!(validator.validate_method("invalid_method", &params).is_err());
    }

    #[test]
    fn test_getblock_wrong_params() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::Number(serde_json::Number::from(123)),
        ]));
        assert!(validator.validate_method("getblock", &params).is_err());
    }

    #[test]
    fn test_fundrawtransaction_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("hexstring".to_string()),
            Value::Array(vec![]),
            Value::String("changeaddress".to_string()),
            Value::Number(serde_json::Number::from(1)), // Using integer instead of float
        ]));
        assert!(validator.validate_method("fundrawtransaction", &params).is_ok());
    }

    #[test]
    fn test_signdata_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::Object(serde_json::Map::new()),
        ]));
        assert!(validator.validate_method("signdata", &params).is_ok());
    }

    #[test]
    fn test_cache_functionality() {
        let mut validator = ComprehensiveValidator::new();
        let (cache_size, cached_methods) = validator.get_cache_stats();
        
        // Should have cached some methods
        assert!(cache_size > 0);
        assert!(cached_methods.contains(&"getinfo".to_string()));
        assert!(cached_methods.contains(&"getblock".to_string()));
        
        // Test cache clearing
        validator.clear_cache();
        let (new_cache_size, _) = validator.get_cache_stats();
        assert_eq!(new_cache_size, 0);
    }

    #[test]
    fn test_number_variant_usage() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("VRSC".to_string()),
            Value::Array(vec![]),
            Value::Number(serde_json::Number::from(123)), // Integer
            Value::Number(serde_json::Number::from_f64(123.45).unwrap()), // Float
            Value::Bool(true), // Boolean
        ]));
        assert!(validator.validate_method("sendcurrency", &params).is_ok());
    }

    #[test]
    fn test_make_offer_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("VRSC".to_string()),
            Value::Object(serde_json::Map::new()),
            Value::String("VRSC".to_string()),
            Value::String("BTC".to_string()),
            Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
            Value::Number(serde_json::Number::from_f64(0.001).unwrap()),
        ]));
        assert!(validator.validate_method("makeOffer", &params).is_ok());
    }

    #[test]
    fn test_make_offer_with_expiry() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("VRSC".to_string()),
            Value::Object(serde_json::Map::new()),
            Value::String("VRSC".to_string()),
            Value::String("BTC".to_string()),
            Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
            Value::Number(serde_json::Number::from_f64(0.001).unwrap()),
            Value::Number(serde_json::Number::from(1640995200)), // Unix timestamp
        ]));
        assert!(validator.validate_method("makeOffer", &params).is_ok());
    }

    #[test]
    fn test_make_offer_invalid_params() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("VRSC".to_string()),
            // Missing required parameters
        ]));
        assert!(validator.validate_method("makeOffer", &params).is_err());
    }

    #[test]
    fn test_z_getnewaddress_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("sapling".to_string()),
        ]));
        assert!(validator.validate_method("z_getnewaddress", &params).is_ok());
    }

    #[test]
    fn test_z_listaddresses_validation() {
        let validator = ComprehensiveValidator::new();
        let params: Option<Value> = None;
        assert!(validator.validate_method("z_listaddresses", &params).is_ok());
    }

    #[test]
    fn test_z_getbalance_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
            Value::Number(serde_json::Number::from(1)),
        ]));
        assert!(validator.validate_method("z_getbalance", &params).is_ok());
    }

    #[test]
    fn test_z_sendmany_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
            Value::Array(vec![
                Value::Object(serde_json::Map::from_iter(vec![
                    ("address".to_string(), Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string())),
                    ("amount".to_string(), Value::Number(serde_json::Number::from_f64(0.1).unwrap())),
                ])),
            ]),
            Value::Number(serde_json::Number::from(1)),
            Value::Number(serde_json::Number::from_f64(0.0001).unwrap()),
        ]));
        assert!(validator.validate_method("z_sendmany", &params).is_ok());
    }

    #[test]
    fn test_z_shieldcoinbase_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("RTestAddress123456789012345678901234567890".to_string()),
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
            Value::Number(serde_json::Number::from_f64(0.0001).unwrap()),
            Value::Number(serde_json::Number::from(100)),
        ]));
        assert!(validator.validate_method("z_shieldcoinbase", &params).is_ok());
    }

    #[test]
    fn test_z_validateaddress_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
        ]));
        assert!(validator.validate_method("z_validateaddress", &params).is_ok());
    }

    #[test]
    fn test_z_viewtransaction_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
        ]));
        assert!(validator.validate_method("z_viewtransaction", &params).is_ok());
    }

    #[test]
    fn test_z_exportkey_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
        ]));
        assert!(validator.validate_method("z_exportkey", &params).is_ok());
    }

    #[test]
    fn test_z_importkey_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("secret-extended-key-main1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string()),
            Value::String("yes".to_string()),
        ]));
        assert!(validator.validate_method("z_importkey", &params).is_ok());
    }

    #[test]
    fn test_z_exportviewingkey_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zs1gqtfu59z20s9t20mxlxj88p5a9hc4f54mrelq9f980mzljpn2rr8r7mx7m2uqqzunfwfmvq9mvz".to_string()),
        ]));
        assert!(validator.validate_method("z_exportviewingkey", &params).is_ok());
    }

    #[test]
    fn test_z_importviewingkey_validation() {
        let validator = ComprehensiveValidator::new();
        let params = Some(Value::Array(vec![
            Value::String("zxviews1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string()),
            Value::String("yes".to_string()),
        ]));
        assert!(validator.validate_method("z_importviewingkey", &params).is_ok());
    }
} 