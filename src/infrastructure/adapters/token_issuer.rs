//! Token issuer adapter for JWT token generation and validation
//! 
//! This adapter handles secure JWT token issuance for external authentication services.

use crate::shared::error::AppResult;
use crate::config::AppConfig;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// JWT claims structure
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

/// Request for token issuance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIssuanceRequest {
    /// User ID (optional for anonymous users)
    pub user_id: String,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Client IP
    pub client_ip: Option<String>,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Custom expiration time (optional, overrides default)
    pub custom_expiration: Option<u64>,
}

/// Response for token issuance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIssuanceResponse {
    /// JWT token
    pub token: String,
    
    /// Token type
    pub token_type: String,
    
    /// Expiration time
    pub expires_in: u64,
    
    /// Token ID
    pub token_id: String,
    
    /// User ID (generated for anonymous users)
    pub user_id: Option<String>,
}

/// Token validation request
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidationRequest {
    /// JWT token to validate
    pub token: String,
    
    /// Client IP (for additional validation)
    pub client_ip: Option<String>,
}

/// Token validation response
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidationResponse {
    /// Whether token is valid
    pub valid: bool,
    
    /// User ID (if valid)
    pub user_id: Option<String>,
    
    /// Permissions (if valid)
    pub permissions: Option<Vec<String>>,
    
    /// Error message (if invalid)
    pub error: Option<String>,
}

/// Token issuer adapter
pub struct TokenIssuerAdapter {
    config: Arc<AppConfig>,
}

impl TokenIssuerAdapter {
    /// Create a new token issuer adapter
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Issue a JWT token
    pub async fn issue_token(&self, request: TokenIssuanceRequest) -> AppResult<TokenIssuanceResponse> {
        info!("Processing token issuance request");
        
        // Validate request
        self.validate_issuance_request(&request).await?;
        
        // Generate token ID
        let token_id = Uuid::new_v4().to_string();
        
        // Handle anonymous users (no user_id provided)
        let user_id = if request.user_id.is_empty() {
            format!("anon_user_{}", Uuid::new_v4().to_string().split('-').next().unwrap())
        } else {
            request.user_id.clone()
        };
        
        let expiration_seconds = request.custom_expiration.unwrap_or(self.config.security.jwt.expiration_seconds);
        let now = Utc::now();
        let expiration = now + Duration::seconds(expiration_seconds as i64);

        let permissions = request.permissions.clone();
        let client_ip = request.client_ip.clone();
        let user_agent = request.user_agent.clone();
        
        let claims = JwtClaims {
            sub: user_id.clone(),
            iss: self.config.security.jwt.issuer.clone(),
            aud: self.config.security.jwt.audience.clone(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
            nbf: now.timestamp() as usize,
            jti: token_id.clone(),
            permissions,
            client_ip,
            user_agent,
        };
        
        // Encode JWT token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.security.jwt.secret_key.as_ref())
        ).map_err(|e| {
            error!("JWT encoding failed: {}", e);
            crate::shared::error::AppError::Internal(format!("Token generation failed: {}", e))
        })?;
        
        info!("JWT token issued successfully for user: {}", user_id);
        
        Ok(TokenIssuanceResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: expiration_seconds,
            token_id,
            user_id: Some(user_id), // Return the generated user ID
        })
    }

    /// Validate a JWT token
    pub async fn validate_token(&self, request: TokenValidationRequest) -> AppResult<TokenValidationResponse> {
        info!("Validating JWT token");
        
        // Decode and validate JWT token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.config.security.jwt.audience]);
        validation.set_issuer(&[&self.config.security.jwt.issuer]);
        
        match decode::<JwtClaims>(
            &request.token,
            &DecodingKey::from_secret(self.config.security.jwt.secret_key.as_ref()),
            &validation
        ) {
            Ok(token_data) => {
                let claims = token_data.claims;
                
                // Check if token is expired
                let current_time = Utc::now().timestamp() as usize;
                if claims.exp < current_time {
                    return Ok(TokenValidationResponse {
                        valid: false,
                        user_id: None,
                        permissions: None,
                        error: Some("Token expired".to_string()),
                    });
                }
                
                // Check if token is not yet valid
                if claims.nbf > current_time {
                    return Ok(TokenValidationResponse {
                        valid: false,
                        user_id: None,
                        permissions: None,
                        error: Some("Token not yet valid".to_string()),
                    });
                }
                
                // Optional: Validate client IP if provided
                if let (Some(token_ip), Some(request_ip)) = (&claims.client_ip, &request.client_ip) {
                    if token_ip != request_ip {
                        warn!("Client IP mismatch: token IP {} vs request IP {}", token_ip, request_ip);
                        // This could be a security warning but not necessarily an error
                    }
                }
                
                info!("JWT token validated successfully for user: {}", claims.sub);
                
                Ok(TokenValidationResponse {
                    valid: true,
                    user_id: Some(claims.sub),
                    permissions: Some(claims.permissions),
                    error: None,
                })
            }
            Err(e) => {
                warn!("JWT validation failed: {}", e);
                Ok(TokenValidationResponse {
                    valid: false,
                    user_id: None,
                    permissions: None,
                    error: Some(format!("Token validation failed: {}", e)),
                })
            }
        }
    }

    /// Validate issuance request
    async fn validate_issuance_request(&self, request: &TokenIssuanceRequest) -> AppResult<()> {
        // User ID is optional for anonymous users
        // if request.user_id.is_empty() {
        //     return Err(crate::shared::error::AppError::Validation("User ID cannot be empty".to_string()));
        // }
        
        // Validate permissions
        if request.permissions.is_empty() {
            return Err(crate::shared::error::AppError::Validation("At least one permission is required".to_string()));
        }
        
        // Validate custom expiration
        if let Some(exp) = request.custom_expiration {
            if exp < 60 || exp > 86400 {
                return Err(crate::shared::error::AppError::Validation("Custom expiration must be between 60 and 86400 seconds".to_string()));
            }
        }
        
        Ok(())
    }

    /// Extract token from authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Option<String> {
        if auth_header.starts_with("Bearer ") {
            Some(auth_header[7..].to_string())
        } else {
            None
        }
    }
}

impl Default for TokenIssuerAdapter {
    fn default() -> Self {
        Self::new(Arc::new(AppConfig::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_token_issuance() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        let request = TokenIssuanceRequest {
            user_id: "test_user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            custom_expiration: None,
        };
        
        let response = issuer.issue_token(request).await.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
        assert!(response.user_id.is_some());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Issue a token
        let issuance_request = TokenIssuanceRequest {
            user_id: "test_user".to_string(),
            permissions: vec!["read".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            custom_expiration: None,
        };
        
        let issuance_response = issuer.issue_token(issuance_request).await.unwrap();
        
        // Validate the token
        let validation_request = TokenValidationRequest {
            token: issuance_response.token,
            client_ip: Some("127.0.0.1".to_string()),
        };
        
        let validation_response = issuer.validate_token(validation_request).await.unwrap();
        assert!(validation_response.valid);
        assert_eq!(validation_response.user_id, Some("test_user".to_string()));
        assert_eq!(validation_response.permissions, Some(vec!["read".to_string()]));
    }

    #[tokio::test]
    async fn test_invalid_token_validation() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        let request = TokenValidationRequest {
            token: "invalid.token.here".to_string(),
            client_ip: None,
        };
        
        let response = issuer.validate_token(request).await.unwrap();
        assert!(!response.valid);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_anonymous_token_issuance() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Test anonymous token issuance (no user_id)
        let issuance_request = TokenIssuanceRequest {
            user_id: "".to_string(), // Empty for anonymous
            permissions: vec!["read".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            custom_expiration: None,
        };
        
        let issuance_response = issuer.issue_token(issuance_request).await.unwrap();
        
        // Verify anonymous user ID was generated
        assert!(issuance_response.user_id.is_some());
        let user_id = issuance_response.user_id.unwrap();
        assert!(user_id.starts_with("anon_user_"));
        assert_eq!(issuance_response.token_type, "Bearer");
        assert_eq!(issuance_response.expires_in, 3600);
        
        // Validate the anonymous token
        let validation_request = TokenValidationRequest {
            token: issuance_response.token,
            client_ip: Some("127.0.0.1".to_string()),
        };
        
        let validation_response = issuer.validate_token(validation_request).await.unwrap();
        assert!(validation_response.valid);
        assert_eq!(validation_response.user_id, Some(user_id));
        assert_eq!(validation_response.permissions, Some(vec!["read".to_string()]));
    }
}
