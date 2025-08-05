// Security module for security-related functionality

use crate::error::AppResult;
use rand::Rng;

/// Security utilities
pub struct SecurityUtils;

impl SecurityUtils {
    /// Generate a secure random token
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let token: u64 = rng.gen();
        format!("{:016x}", token)
    }

    /// Generate a random salt
    pub fn generate_salt() -> String {
        let mut rng = rand::thread_rng();
        let salt: u64 = rng.gen();
        format!("{:016x}", salt)
    }

    /// Simple string validation
    pub fn validate_input(input: &str) -> AppResult<()> {
        if input.is_empty() {
            return Err(crate::error::AppError::Validation("Input cannot be empty".to_string()));
        }
        
        if input.len() > 1000 {
            return Err(crate::error::AppError::Validation("Input too long".to_string()));
        }
        
        Ok(())
    }
} 