//! Mining pool utilities module
//! 
//! This module contains mining pool specific utility functions.

use crate::shared::error::AppResult;
use crate::domain::rpc::RpcRequest;
use crate::infrastructure::adapters::PoolShare;
use crate::shared::error::AppError;
use serde_json::Value;

/// Mining pool utilities
pub struct MiningPoolUtils;

impl MiningPoolUtils {
    /// Parse pool share from domain request parameters
    pub fn parse_pool_share_from_request(domain_request: &RpcRequest) -> AppResult<PoolShare> {
        // Extract parameters from the domain request
        let params = domain_request.parameters.as_ref()
            .ok_or_else(|| AppError::Validation("Missing request parameters".to_string()))?;
        
        // Parse the parameters as a JSON object
        let params_obj = params.as_object()
            .ok_or_else(|| AppError::Validation("Parameters must be a JSON object".to_string()))?;
        
        // Extract required fields with validation
        let challenge_id = params_obj.get("challenge_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid challenge_id".to_string()))?
            .to_string();
        
        let miner_address = params_obj.get("miner_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid miner_address".to_string()))?
            .to_string();
        
        let nonce = params_obj.get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid nonce".to_string()))?
            .to_string();
        
        let solution = params_obj.get("solution")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid solution".to_string()))?
            .to_string();
        
        let difficulty = params_obj.get("difficulty")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Validation("Missing or invalid difficulty".to_string()))?;
        
        // Parse timestamp (accept both ISO string and Unix timestamp)
        let timestamp = if let Some(timestamp_value) = params_obj.get("timestamp") {
            match timestamp_value {
                Value::String(timestamp_str) => {
                    // Try to parse as ISO 8601 string
                    chrono::DateTime::parse_from_rfc3339(timestamp_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .or_else(|_| {
                            // Try to parse as Unix timestamp
                            timestamp_str.parse::<i64>()
                                .ok()
                                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                                .ok_or_else(|| AppError::Validation("Invalid timestamp format".to_string()))
                        })?
                }
                Value::Number(timestamp_num) => {
                    // Parse as Unix timestamp
                    timestamp_num.as_i64()
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .ok_or_else(|| AppError::Validation("Invalid timestamp value".to_string()))?
                }
                _ => return Err(AppError::Validation("Invalid timestamp format".to_string())),
            }
        } else {
            // Use current timestamp if not provided
            chrono::Utc::now()
        };
        
        // Parse optional pool signature
        let pool_signature = params_obj.get("pool_signature")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Validate field lengths and values
        if challenge_id.is_empty() {
            return Err(AppError::Validation("challenge_id cannot be empty".to_string()));
        }
        if miner_address.is_empty() {
            return Err(AppError::Validation("miner_address cannot be empty".to_string()));
        }
        if nonce.is_empty() {
            return Err(AppError::Validation("nonce cannot be empty".to_string()));
        }
        if solution.is_empty() {
            return Err(AppError::Validation("solution cannot be empty".to_string()));
        }
        if difficulty <= 0.0 {
            return Err(AppError::Validation("difficulty must be positive".to_string()));
        }
        
        Ok(PoolShare {
            challenge_id,
            miner_address,
            nonce,
            solution,
            difficulty,
            timestamp,
            pool_signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rpc::RpcRequest;

    fn create_test_domain_request() -> RpcRequest {
        RpcRequest {
            method: "submit_share".to_string(),
            parameters: Some(serde_json::json!({
                "challenge_id": "test_challenge_123",
                "miner_address": "test_miner_address",
                "nonce": "test_nonce_456",
                "solution": "test_solution_789",
                "difficulty": 1.5,
                "timestamp": "2023-01-01T00:00:00Z",
                "pool_signature": "test_signature"
            })),
            id: Some(serde_json::json!(1)),
            client_info: crate::domain::rpc::ClientInfo {
                ip_address: "127.0.0.1".to_string(),
                user_agent: Some("test_agent".to_string()),
                timestamp: chrono::Utc::now(),
            },
        }
    }

    #[test]
    fn test_parse_pool_share_from_request_success() {
        let domain_request = create_test_domain_request();
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_ok());
        
        let pool_share = result.unwrap();
        assert_eq!(pool_share.challenge_id, "test_challenge_123");
        assert_eq!(pool_share.miner_address, "test_miner_address");
        assert_eq!(pool_share.nonce, "test_nonce_456");
        assert_eq!(pool_share.solution, "test_solution_789");
        assert_eq!(pool_share.difficulty, 1.5);
        assert_eq!(pool_share.pool_signature, Some("test_signature".to_string()));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_parameters() {
        let mut domain_request = create_test_domain_request();
        domain_request.parameters = None;
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing request parameters"));
    }

    #[test]
    fn test_parse_pool_share_from_request_invalid_parameters_format() {
        let mut domain_request = create_test_domain_request();
        domain_request.parameters = Some(serde_json::json!("invalid_format"));
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parameters must be a JSON object"));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_challenge_id() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("challenge_id");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing or invalid challenge_id"));
    }

    #[test]
    fn test_parse_pool_share_from_request_empty_challenge_id() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.insert("challenge_id".to_string(), serde_json::json!(""));
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("challenge_id cannot be empty"));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_miner_address() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("miner_address");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing or invalid miner_address"));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_nonce() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("nonce");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing or invalid nonce"));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_solution() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("solution");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing or invalid solution"));
    }

    #[test]
    fn test_parse_pool_share_from_request_missing_difficulty() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("difficulty");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing or invalid difficulty"));
    }

    #[test]
    fn test_parse_pool_share_from_request_invalid_difficulty() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.insert("difficulty".to_string(), serde_json::json!(0.0));
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("difficulty must be positive"));
    }

    #[test]
    fn test_parse_pool_share_from_request_with_unix_timestamp() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.insert("timestamp".to_string(), serde_json::json!(1640995200)); // Unix timestamp
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pool_share_from_request_without_timestamp() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("timestamp");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_ok());
        let pool_share = result.unwrap();
        // Should use current timestamp
        assert!(pool_share.timestamp > chrono::Utc::now() - chrono::Duration::seconds(5));
    }

    #[test]
    fn test_parse_pool_share_from_request_without_pool_signature() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.remove("pool_signature");
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_ok());
        let pool_share = result.unwrap();
        assert_eq!(pool_share.pool_signature, None);
    }

    #[test]
    fn test_parse_pool_share_from_request_invalid_timestamp_format() {
        let mut domain_request = create_test_domain_request();
        if let Some(params) = domain_request.parameters.as_mut() {
            if let Some(params_obj) = params.as_object_mut() {
                params_obj.insert("timestamp".to_string(), serde_json::json!("invalid_timestamp"));
            }
        }
        
        let result = MiningPoolUtils::parse_pool_share_from_request(&domain_request);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid timestamp format"));
    }
}
