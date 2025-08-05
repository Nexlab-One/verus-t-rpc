use crate::error::{AppError, AppResult};
use serde_json::{Value, value::RawValue};

/// Method validator for RPC requests
pub struct MethodValidator;

impl MethodValidator {
    /// Create a new method validator
    pub fn new() -> Self {
        Self
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
        match method {
            "fundrawtransaction" => self.validate_fundrawtransaction(params),
            "signdata" => self.validate_signdata(params),
            "recoveridentity" => self.validate_recoveridentity(params),
            "registeridentity" => self.validate_registeridentity(params),
            "revokeidentity" => self.validate_revokeidentity(params),
            "updateidentity" => self.validate_updateidentity(params),
            "setidentitytimelock" => self.validate_setidentitytimelock(params),
            "sendcurrency" => self.validate_sendcurrency(params),
            "coinsupply" => self.check_params(params, &[]),
            "convertpassphrase" => self.check_params(params, &["str"]),
            "createmultisig" => self.check_params(params, &["int", "arr"]),
            "createrawtransaction" => self.check_params(params, &["arr", "obj", "int", "int"]),
            "decoderawtransaction" => self.check_params(params, &["str", "bool"]),
            "decodescript" => self.check_params(params, &["str", "bool"]),
            "estimateconversion" => self.check_params(params, &["obj"]),
            "estimatefee" => self.check_params(params, &["int"]),
            "estimatepriority" => self.check_params(params, &["int"]),
            "getaddressmempool" => self.check_params(params, &["obj"]),
            "getaddressutxos" => self.check_params(params, &["obj"]),
            "getaddressbalance" => self.check_params(params, &["obj"]),
            "getaddressdeltas" => self.check_params(params, &["obj"]),
            "getaddresstxids" => self.check_params(params, &["obj"]),
            "getbestblockhash" => self.check_params(params, &[]),
            "getbestproofroot" => self.check_params(params, &["obj"]),
            "getblock" => self.check_params(params, &["str", "bool"]),
            "getblockchaininfo" => self.check_params(params, &[]),
            "getblockcount" => self.check_params(params, &[]),
            "getblockhashes" => self.check_params(params, &["int", "int"]),
            "getblockhash" => self.check_params(params, &["int"]),
            "getblockheader" => self.check_params(params, &["str"]),
            "getblocksubsidy" => self.check_params(params, &["int"]),
            "getblocktemplate" => self.check_params(params, &["obj"]),
            "getchaintips" => self.check_params(params, &[]),
            "getcurrency" => self.check_params(params, &["str"]),
            "getcurrencyconverters" => self.check_params(params, &["str", "str", "str"]),
            "getcurrencystate" => self.check_params(params, &["str", "str", "str"]),
            "getcurrencytrust" => self.check_params(params, &["arr"]),
            "getdifficulty" => self.check_params(params, &[]),
            "getexports" => self.check_params(params, &["str", "int", "int"]),
            "getinfo" => self.check_params(params, &[]),
            "getinitialcurrencystate" => self.check_params(params, &["str"]),
            "getidentitieswithaddress" => self.check_params(params, &["obj"]),
            "getidentitieswithrevocation" => self.check_params(params, &["obj"]),
            "getidentitieswithrecovery" => self.check_params(params, &["obj"]),
            "getidentity" => self.check_params(params, &["str", "int", "bool", "int"]),
            "getidentitytrust" => self.check_params(params, &["arr"]),
            "getidentitycontent" => self.check_params(params, &["str", "int", "int", "bool", "int", "str", "bool"]),
            "getlastimportfrom" => self.check_params(params, &["str"]),
            "getlaunchinfo" => self.check_params(params, &["str"]),
            "getmempoolinfo" => self.check_params(params, &[]),
            "getmininginfo" => self.check_params(params, &[]),
            "getnetworkinfo" => self.check_params(params, &[]),
            "getnotarizationdata" => self.check_params(params, &["str"]),
            "getoffers" => self.check_params(params, &["str", "bool", "bool"]),
            "getpendingtransfers" => self.check_params(params, &["str"]),
            "getrawmempool" => self.check_params(params, &[]),
            "getrawtransaction" => self.check_params(params, &["str", "int"]),
            "getreservedeposits" => self.check_params(params, &["str"]),
            "getsaplingtree" => self.check_params(params, &["int"]),
            "getspentinfo" => self.check_params(params, &["obj"]),
            "gettxout" => self.check_params(params, &["str", "int", "bool"]),
            "gettxoutsetinfo" => self.check_params(params, &[]),
            "getvdxfid" => self.check_params(params, &["str", "obj"]),
            "hashdata" => self.check_params(params, &["str", "str", "str"]),
            "help" => self.check_params(params, &[]),
            "listcurrencies" => self.check_params(params, &["obj", "int", "int"]),
            "sendrawtransaction" => self.check_params(params, &["str"]),
            "submitacceptednotarization" => self.check_params(params, &["obj", "obj"]),
            "submitimports" => self.check_params(params, &["obj"]),
            "verifymessage" => self.check_params(params, &["str", "str", "str", "bool"]),
            "verifyhash" => self.check_params(params, &["str", "str", "str", "bool"]),
            "verifysignature" => self.check_params(params, &["obj"]),
            _ => false,
        }
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
            && self.check_params(params, &["obj", "bool", "bool", "float", "str"])
    }

    /// Validate registeridentity method
    fn validate_registeridentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &["obj", "bool", "float", "str"])
    }

    /// Validate revokeidentity method
    fn validate_revokeidentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &["str", "bool", "bool", "float", "str"])
    }

    /// Validate updateidentity method
    fn validate_updateidentity(&self, params: &[Box<RawValue>]) -> bool {
        params.get(1)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &["obj", "bool", "bool", "float", "str"])
    }

    /// Validate setidentitytimelock method
    fn validate_setidentitytimelock(&self, params: &[Box<RawValue>]) -> bool {
        params.get(2)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &["str", "obj", "bool", "float", "str"])
    }

    /// Validate sendcurrency method
    fn validate_sendcurrency(&self, params: &[Box<RawValue>]) -> bool {
        params.get(4)
            .and_then(|p| serde_json::from_str::<Value>(&p.to_string()).ok())
            .map_or(false, |v| v.as_bool().unwrap_or(false)) 
            && self.check_params(params, &["str", "arr", "int", "float", "bool"])
    }

    /// Check parameter types against expected types
    fn check_params(&self, params: &[Box<RawValue>], expected_types: &[&str]) -> bool {
        if params.len() > expected_types.len() {
            return false;
        }
        
        for (param, &expected_type) in params.iter().zip(expected_types) {
            let value: Value = match serde_json::from_str(&param.to_string()) {
                Ok(v) => v,
                Err(_) => return false,
            };
            
            match expected_type {
                "obj" => if !matches!(value, Value::Object(_)) { return false; },
                "arr" => if !matches!(value, Value::Array(_)) { return false; },
                "int" => if !matches!(value, Value::Number(n) if n.is_i64()) { return false; },
                "float" => if !matches!(value, Value::Number(n) if n.is_f64()) { return false; },
                "str" => if !matches!(value, Value::String(_)) { return false; },
                "bool" => if !matches!(value, Value::Bool(_)) { return false; },
                _ => return false,
            }
        }
        
        true
    }
} 