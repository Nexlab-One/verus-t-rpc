//! Authentication adapter for token validation
//! 
//! This adapter handles JWT token validation and user permission management.

use crate::shared::error::AppResult;
use crate::config::AppConfig;
use std::sync::Arc;
use tracing::{info, warn, error};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// JWT claims structure for validation
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Issuer
    pub iss: String,
    
    /// Audience
    pub aud: String,
    
    /// Issued at
    pub iat: usize,
    
    /// Expiration time
    pub exp: usize,
    
    /// Not before
    pub nbf: usize,
    
    /// JWT ID (unique identifier)
    pub jti: String,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Client IP (for additional security)
    pub client_ip: Option<String>,
    
    /// User agent (for additional security)
    pub user_agent: Option<String>,
}

/// Adapter for authentication services
pub struct AuthenticationAdapter {
    config: Arc<AppConfig>,
}

impl AuthenticationAdapter {
    /// Create a new authentication adapter
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Validate authentication token
    pub async fn validate_token(&self, token: &str) -> AppResult<Vec<String>> {
        info!("Validating authentication token");
        
        if token.is_empty() {
            return Err(crate::shared::error::AppError::Authentication("Empty token".to_string()));
        }

        // Validate token format
        if !token.starts_with("Bearer ") {
            return Err(crate::shared::error::AppError::Authentication("Invalid token format".to_string()));
        }

        let token_value = &token[7..]; // Remove "Bearer " prefix
        
        // Basic token validation
        if token_value.len() < 10 {
            return Err(crate::shared::error::AppError::Authentication("Token too short".to_string()));
        }

        // Validate as JWT token
        self.validate_jwt_token(token_value).await
    }

    /// Validate JWT token
    async fn validate_jwt_token(&self, token: &str) -> AppResult<Vec<String>> {
        // Decode and validate JWT token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.config.security.jwt.audience]);
        validation.set_issuer(&[&self.config.security.jwt.issuer]);
        
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.security.jwt.secret_key.as_ref()),
            &validation
        ).map_err(|e| {
            error!("JWT validation failed: {}", e);
            crate::shared::error::AppError::Authentication(format!("JWT validation failed: {}", e))
        })?;

        let claims = token_data.claims;
        
        // Check if token is expired
        let current_time = Utc::now().timestamp() as usize;
        if claims.exp < current_time {
            return Err(crate::shared::error::AppError::Authentication("Token expired".to_string()));
        }
        
        // Check if token is not yet valid
        if claims.nbf > current_time {
            return Err(crate::shared::error::AppError::Authentication("Token not yet valid".to_string()));
        }

        // Extract permissions from token
        let permissions = claims.permissions;
        
        if permissions.is_empty() {
            warn!("Token has no permissions for user: {}", claims.sub);
            return Ok(vec!["read".to_string()]); // Default to read-only
        }
        
        info!("JWT token validated successfully for user: {} with permissions: {:?}", claims.sub, permissions);
        
        Ok(permissions)
    }

    /// Extract token from request headers
    pub fn extract_token(&self, headers: &str) -> Option<String> {
        // Parse headers string and look for Authorization header
        for line in headers.lines() {
            if line.starts_with("Authorization:") {
                let token = line[14..].trim(); // Remove "Authorization: " prefix and trim whitespace
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
        None
    }

    /// Extract token from authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Option<String> {
        if auth_header.starts_with("Bearer ") {
            Some(auth_header.to_string())
        } else {
            None
        }
    }
}

impl Default for AuthenticationAdapter {
    fn default() -> Self {
        Self::new(Arc::new(AppConfig::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_token_validation() {
        let config = Arc::new(AppConfig::default());
        let auth = AuthenticationAdapter::new(config);
        
        // Test empty token
        let result = auth.validate_token("").await;
        assert!(result.is_err());
        
        // Test invalid format
        let result = auth.validate_token("invalid").await;
        assert!(result.is_err());
        
        // Test too short token
        let result = auth.validate_token("Bearer short").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_extraction() {
        let config = Arc::new(AppConfig::default());
        let auth = AuthenticationAdapter::new(config);
        
        // Test valid header
        let headers = "Authorization: Bearer valid.token.here\nContent-Type: application/json";
        let token = auth.extract_token(headers);
        assert_eq!(token, Some("Bearer valid.token.here".to_string()));
        
        // Test missing header
        let headers = "Content-Type: application/json";
        let token = auth.extract_token(headers);
        assert_eq!(token, None);
        
        // Test with extra spaces
        let headers = "Authorization:  Bearer valid.token.here  \nContent-Type: application/json";
        let token = auth.extract_token(headers);
        assert_eq!(token, Some("Bearer valid.token.here".to_string()));
    }
} 