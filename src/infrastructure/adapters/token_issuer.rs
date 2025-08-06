//! Token issuer adapter for JWT token generation and validation
//! 
//! This adapter handles secure JWT token issuance for external authentication services.

use crate::shared::error::AppResult;
use crate::config::AppConfig;
use std::sync::Arc;
use tracing::{info, warn, error};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use blake3::Hasher;
use crate::infrastructure::adapters::mining_pool::{PoolShare, MiningPoolClient};

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

/// Token issuance mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenIssuanceMode {
    /// Anonymous token (current implementation)
    Anonymous,
    /// PoW-validated token with enhanced permissions
    ProofOfWork(PowProof),
    /// Pool-validated token with enhanced permissions
    PoolValidated(PoolShare),
    /// Partner-issued token (for trusted DEXs)
    Partner(String),
}

/// PoW algorithm types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PowAlgorithm {
    Sha256,
    Blake3,
}

/// PoW challenge structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowChallenge {
    /// Unique challenge identifier
    pub id: String,
    
    /// Challenge string to hash
    pub challenge: String,
    
    /// Target difficulty (hex string)
    pub target_difficulty: String,
    
    /// Algorithm to use
    pub algorithm: PowAlgorithm,
    
    /// When challenge expires
    pub expires_at: chrono::DateTime<Utc>,
    
    /// Token duration if solved (seconds)
    pub token_duration: u64,
    
    /// Rate limit multiplier
    pub rate_limit_multiplier: f64,
}

/// PoW proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowProof {
    /// Challenge ID this proof solves
    pub challenge_id: String,
    
    /// Nonce used
    pub nonce: String,
    
    /// Solution hash
    pub solution: String,
    
    /// Actual difficulty achieved
    pub difficulty: String,
    
    /// When proof was submitted
    pub submitted_at: chrono::DateTime<Utc>,
    
    /// Client IP that solved it
    pub client_ip: String,
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
    
    /// Issuance mode
    pub mode: TokenIssuanceMode,
    
    /// PoW challenge (if requesting PoW mode)
    pub pow_challenge: Option<PowChallenge>,
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

/// Adapter for token issuance and validation
pub struct TokenIssuerAdapter {
    config: Arc<AppConfig>,
    pub pow_manager: PowManager,
    pub mining_pool_client: Option<MiningPoolClient>,
}

impl TokenIssuerAdapter {
    /// Create a new token issuer adapter
    pub fn new(config: Arc<AppConfig>) -> Self {
        let mining_pool_client = if config.security.mining_pool.is_some() {
            Some(MiningPoolClient::new(config.clone()))
        } else {
            None
        };
        
        Self {
            config: config.clone(),
            pow_manager: PowManager::new(config),
            mining_pool_client,
        }
    }

    /// Issue a JWT token
    pub async fn issue_token(&self, request: TokenIssuanceRequest) -> AppResult<TokenIssuanceResponse> {
        info!("Processing token issuance request");
        
        // Validate request
        self.validate_issuance_request(&request).await?;
        
        // Handle different issuance modes
        match &request.mode {
            TokenIssuanceMode::Anonymous => {
                self.issue_anonymous_token(request).await
            }
            TokenIssuanceMode::ProofOfWork(proof) => {
                self.issue_pow_token(&request, proof).await
            }
            TokenIssuanceMode::PoolValidated(share) => {
                self.issue_pool_token(&request, share).await
            }
            TokenIssuanceMode::Partner(partner_id) => {
                self.issue_partner_token(&request, partner_id).await
            }
        }
    }
    
    /// Issue anonymous token (current implementation)
    async fn issue_anonymous_token(&self, request: TokenIssuanceRequest) -> AppResult<TokenIssuanceResponse> {
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
            user_id: Some(user_id),
        })
    }
    
    /// Issue PoW-validated token
    async fn issue_pow_token(
        &self, 
        request: &TokenIssuanceRequest, 
        proof: &PowProof
    ) -> AppResult<TokenIssuanceResponse> {
        info!("Processing PoW token issuance request");
        
        // Get the challenge from the request
        let challenge = request.pow_challenge.as_ref().ok_or_else(|| {
            crate::shared::error::AppError::Validation("PoW challenge required for PoW token issuance".to_string())
        })?;
        
        // Validate PoW proof
        let is_valid = self.pow_manager.verify_solution(challenge, proof).await?;
        if !is_valid {
            return Err(crate::shared::error::AppError::Validation("Invalid PoW proof".to_string()));
        }
        
        // Create enhanced token request
        let enhanced_request = TokenIssuanceRequest {
            user_id: request.user_id.clone(),
            permissions: self.enhance_permissions(&request.permissions, challenge),
            client_ip: request.client_ip.clone(),
            user_agent: request.user_agent.clone(),
            custom_expiration: Some(challenge.token_duration),
            mode: TokenIssuanceMode::Anonymous, // Convert to anonymous after validation
            pow_challenge: None,
        };
        
        // Issue token with enhanced privileges
        self.issue_anonymous_token(enhanced_request).await
    }
    
    /// Issue Pool-validated token
    async fn issue_pool_token(
        &self, 
        request: &TokenIssuanceRequest, 
        share: &PoolShare
    ) -> AppResult<TokenIssuanceResponse> {
        info!("Processing Pool-validated token issuance request");
        
        // Validate the share with the mining pool
        let pool_client = self.mining_pool_client.as_ref()
            .ok_or_else(|| crate::shared::error::AppError::Internal(
                "Mining pool client not available".to_string()
            ))?;
        
        let validation_response = pool_client.validate_share(share).await?;
        
        if !validation_response.valid {
            return Err(crate::shared::error::AppError::Validation(
                validation_response.error.unwrap_or_else(|| 
                    "Pool share validation failed".to_string()
                )
            ));
        }
        
        info!("Pool share validated successfully: share_id={:?}, reputation={:?}",
              validation_response.share_id, validation_response.miner_reputation);
        
        // Enhance permissions based on the validated share
        let enhanced_request = TokenIssuanceRequest {
            user_id: request.user_id.clone(),
            permissions: self.enhance_pool_permissions(&request.permissions, share),
            client_ip: request.client_ip.clone(),
            user_agent: request.user_agent.clone(),
            custom_expiration: Some(3600 * 24), // 24 hours for pool shares
            mode: TokenIssuanceMode::Anonymous, // Convert to anonymous after validation
            pow_challenge: None,
        };
        
        self.issue_anonymous_token(enhanced_request).await
    }
    
    /// Issue partner token (placeholder for future implementation)
    async fn issue_partner_token(
        &self, 
        request: &TokenIssuanceRequest, 
        partner_id: &str
    ) -> AppResult<TokenIssuanceResponse> {
        // For now, treat partner tokens as anonymous with enhanced permissions
        // In production, this would validate partner credentials
        info!("Processing partner token issuance for partner: {}", partner_id);
        
        let enhanced_request = TokenIssuanceRequest {
            user_id: request.user_id.clone(),
            permissions: self.enhance_partner_permissions(&request.permissions, partner_id),
            client_ip: request.client_ip.clone(),
            user_agent: request.user_agent.clone(),
            custom_expiration: Some(3600 * 24), // 24 hours for partners
            mode: TokenIssuanceMode::Anonymous,
            pow_challenge: None,
        };
        
        self.issue_anonymous_token(enhanced_request).await
    }
    
    /// Enhance permissions based on PoW validation
    fn enhance_permissions(&self, base_permissions: &[String], challenge: &PowChallenge) -> Vec<String> {
        let mut enhanced = base_permissions.to_vec();
        
        // Add PoW-specific permissions
        enhanced.push("pow_validated".to_string());
        
        // Add rate limit multiplier permission
        enhanced.push(format!("rate_multiplier_{}", challenge.rate_limit_multiplier));
        
        enhanced
    }
    
    /// Enhance permissions for partner tokens
    fn enhance_partner_permissions(&self, base_permissions: &[String], partner_id: &str) -> Vec<String> {
        let mut enhanced = base_permissions.to_vec();
        
        // Add partner-specific permissions
        enhanced.push("partner_validated".to_string());
        enhanced.push(format!("partner_{}", partner_id));
        enhanced.push("rate_multiplier_3.0".to_string()); // 3x rate limit for partners
        
        enhanced
    }

    /// Enhance permissions for pool-validated tokens
    fn enhance_pool_permissions(&self, base_permissions: &[String], share: &PoolShare) -> Vec<String> {
        let mut enhanced = base_permissions.to_vec();
        
        // Add pool-specific permissions
        enhanced.push("pool_validated".to_string());
        enhanced.push(format!("miner_{}", share.miner_address));
        enhanced.push("rate_multiplier_2.0".to_string()); // 2x rate limit for pools
        
        enhanced
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

    /// Generate a new PoW challenge
    pub async fn generate_pow_challenge(&self, client_ip: &str) -> AppResult<PowChallenge> {
        self.pow_manager.generate_challenge(client_ip).await
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

/// PoW Manager for challenge generation and validation
pub struct PowManager {
    config: Arc<AppConfig>,
}

impl PowManager {
    /// Create a new PoW manager
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Generate new PoW challenge
    pub async fn generate_challenge(&self, _client_ip: &str) -> AppResult<PowChallenge> {
        let difficulty = self.get_current_difficulty().await;
        let challenge_id = Uuid::new_v4().to_string();
        
        // Get configuration values
        let expiration_minutes = if let Some(pow_config) = &self.config.security.pow {
            pow_config.challenge_expiration_minutes
        } else {
            10 // Default
        };
        
        let token_duration = if let Some(pow_config) = &self.config.security.pow {
            pow_config.token_duration_seconds
        } else {
            3600 * 4 // 4 hours default
        };
        
        let rate_multiplier = if let Some(pow_config) = &self.config.security.pow {
            pow_config.rate_limit_multiplier
        } else {
            2.0 // Default
        };
        
        let challenge = PowChallenge {
            id: challenge_id.clone(),
            challenge: format!("verus_rpc_{}_{}", challenge_id, Utc::now().timestamp()),
            target_difficulty: difficulty.clone(),
            algorithm: PowAlgorithm::Sha256, // Start with SHA256
            expires_at: Utc::now() + Duration::minutes(expiration_minutes as i64),
            token_duration,
            rate_limit_multiplier: rate_multiplier,
        };
        
        info!("Generated PoW challenge: {} with difficulty: {}", challenge_id, difficulty);
        Ok(challenge)
    }
    
    /// Verify PoW solution
    pub async fn verify_solution(
        &self, 
        challenge: &PowChallenge, 
        proof: &PowProof
    ) -> AppResult<bool> {
        // Verify challenge ID matches
        if proof.challenge_id != challenge.id {
            warn!("PoW proof challenge ID mismatch: expected {}, got {}", 
                  challenge.id, proof.challenge_id);
            return Ok(false);
        }
        
        // Check if challenge expired
        if Utc::now() > challenge.expires_at {
            warn!("PoW challenge expired: {}", challenge.id);
            return Ok(false);
        }
        
        // Hash the challenge + nonce
        let input = format!("{}{}", challenge.challenge, proof.nonce);
        let hash = match challenge.algorithm {
            PowAlgorithm::Sha256 => self.hash_sha256(&input),
            PowAlgorithm::Blake3 => self.hash_blake3(&input),
        };
        
        // Verify the solution hash matches
        if hash != proof.solution {
            warn!("PoW solution hash mismatch for challenge: {}", challenge.id);
            return Ok(false);
        }
        
        // Check if hash meets target difficulty
        let hash_int = u64::from_str_radix(&hash[..8], 16)
            .map_err(|_| crate::shared::error::AppError::Validation("Invalid hash format".to_string()))?;
        
        let target_int = u64::from_str_radix(&challenge.target_difficulty, 16)
            .map_err(|_| crate::shared::error::AppError::Validation("Invalid difficulty format".to_string()))?;
        
        let is_valid = hash_int <= target_int;
        
        if is_valid {
            info!("PoW solution validated for challenge: {} with difficulty: {}", 
                  challenge.id, proof.difficulty);
        } else {
            warn!("PoW solution failed difficulty check for challenge: {} (hash: {}, target: {})", 
                  challenge.id, hash_int, target_int);
        }
        
        Ok(is_valid)
    }
    
    /// Get current difficulty based on recent solve times
    async fn get_current_difficulty(&self) -> String {
        // For now, return a fixed difficulty
        // In production, this would adjust based on recent solve times
        // and could use configuration values from self.config
        if let Some(pow_config) = &self.config.security.pow {
            pow_config.default_difficulty.clone()
        } else {
            "0000ffff".to_string()
        }
    }
    
    /// Hash input using SHA256
    pub fn hash_sha256(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Hash input using Blake3
    fn hash_blake3(&self, input: &str) -> String {
        let hash = Hasher::new().update(input.as_bytes()).finalize();
        hex::encode(hash.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, app_config::PowConfig};

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
            mode: TokenIssuanceMode::Anonymous,
            pow_challenge: None,
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
            mode: TokenIssuanceMode::Anonymous,
            pow_challenge: None,
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
            mode: TokenIssuanceMode::Anonymous,
            pow_challenge: None,
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

    #[tokio::test]
    async fn test_pow_challenge_generation() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a PoW challenge
        let challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Verify challenge structure
        assert!(!challenge.id.is_empty());
        assert!(challenge.challenge.starts_with("verus_rpc_"));
        assert_eq!(challenge.algorithm, PowAlgorithm::Sha256);
        assert_eq!(challenge.rate_limit_multiplier, 2.0);
        assert_eq!(challenge.token_duration, 3600 * 4); // 4 hours
        
        // Verify challenge hasn't expired
        assert!(Utc::now() < challenge.expires_at);
    }
    
    #[tokio::test]
    async fn test_pow_token_issuance() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a PoW challenge
        let challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Create a valid PoW proof (this would normally be solved by mining)
        let proof = PowProof {
            challenge_id: challenge.id.clone(),
            nonce: "12345678".to_string(),
            solution: "0000abcd1234567890abcdef1234567890abcdef1234567890abcdef12345678".to_string(),
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        // Create PoW token request
        let issuance_request = TokenIssuanceRequest {
            user_id: "pow_user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("TestApp/1.0".to_string()),
            custom_expiration: None,
            mode: TokenIssuanceMode::ProofOfWork(proof),
            pow_challenge: Some(challenge),
        };
        
        // Note: This test will fail because the PoW proof is not actually valid
        // In a real scenario, the proof would be generated by solving the challenge
        let result = issuer.issue_token(issuance_request).await;
        assert!(result.is_err()); // Should fail due to invalid PoW proof
    }
    
    #[tokio::test]
    async fn test_pow_verification() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge
        let challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Create a proof with the actual solution
        let input = format!("{}{}", challenge.challenge, "0");
        let solution = issuer.pow_manager.hash_sha256(&input);
        
        let proof = PowProof {
            challenge_id: challenge.id.clone(),
            nonce: "0".to_string(),
            solution: solution.clone(),
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        // Verify the solution
        let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
        
        // This should be valid if the hash meets the difficulty
        // The actual result depends on the hash value
        println!("PoW verification result: {} (solution: {})", is_valid, solution);
    }
    
    #[tokio::test]
    async fn test_partner_token_issuance() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Create partner token request
        let issuance_request = TokenIssuanceRequest {
            user_id: "partner_user".to_string(),
            permissions: vec!["read".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("PartnerApp/1.0".to_string()),
            custom_expiration: None,
            mode: TokenIssuanceMode::Partner("test_partner".to_string()),
            pow_challenge: None,
        };
        
        let issuance_response = issuer.issue_token(issuance_request).await.unwrap();
        
        // Verify partner token has enhanced permissions
        assert!(issuance_response.user_id.is_some());
        assert_eq!(issuance_response.token_type, "Bearer");
        assert_eq!(issuance_response.expires_in, 3600 * 24); // 24 hours for partners
    }

    #[tokio::test]
    async fn test_pow_challenge_with_config() {
        let mut config = AppConfig::default();
        config.security.pow = Some(PowConfig {
            default_difficulty: "0000aaaa".to_string(),
            challenge_expiration_minutes: 15,
            token_duration_seconds: 7200, // 2 hours
            rate_limit_multiplier: 3.0,
            enabled: true,
        });
        
        let config = Arc::new(config);
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a PoW challenge
        let challenge = issuer.generate_pow_challenge("192.168.1.100").await.unwrap();
        
        // Verify challenge uses config values
        assert_eq!(challenge.target_difficulty, "0000aaaa");
        assert_eq!(challenge.token_duration, 7200);
        assert_eq!(challenge.rate_limit_multiplier, 3.0);
        
        // Verify expiration is set correctly
        let expected_expiration = Utc::now() + Duration::minutes(15);
        assert!(challenge.expires_at > Utc::now());
        assert!(challenge.expires_at <= expected_expiration);
    }
    
    #[tokio::test]
    async fn test_pow_verification_with_valid_solution() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge with very low difficulty for testing
        let mut challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        challenge.target_difficulty = "ffffffff".to_string(); // Very easy difficulty
        
        // Try different nonces until we find a valid solution
        let mut valid_proof = None;
        for nonce in 0..1000 {
            let input = format!("{}{}", challenge.challenge, nonce);
            let solution = issuer.pow_manager.hash_sha256(&input);
            
            // Check if solution meets difficulty (first 8 chars should be <= target for hex comparison)
            if solution[..8] <= challenge.target_difficulty[..8] {
                let proof = PowProof {
                    challenge_id: challenge.id.clone(),
                    nonce: nonce.to_string(),
                    solution: solution.clone(),
                    difficulty: "ffffffff".to_string(),
                    submitted_at: Utc::now(),
                    client_ip: "127.0.0.1".to_string(),
                };
                
                let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
                if is_valid {
                    valid_proof = Some(proof);
                    break;
                }
            }
        }
        
        // We should find a valid proof with such low difficulty
        assert!(valid_proof.is_some());
        println!("Found valid PoW proof with nonce: {}", valid_proof.as_ref().unwrap().nonce);
    }
    
    #[tokio::test]
    async fn test_pow_verification_invalid_challenge_id() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge
        let challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Create proof with wrong challenge ID
        let proof = PowProof {
            challenge_id: "wrong-challenge-id".to_string(),
            nonce: "123".to_string(),
            solution: "0000abcd1234567890abcdef1234567890abcdef1234567890abcdef12345678".to_string(),
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_pow_verification_expired_challenge() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge
        let mut challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Manually set expiration to past
        challenge.expires_at = Utc::now() - Duration::minutes(1);
        
        let proof = PowProof {
            challenge_id: challenge.id.clone(),
            nonce: "123".to_string(),
            solution: "0000abcd1234567890abcdef1234567890abcdef1234567890abcdef12345678".to_string(),
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_pow_verification_hash_mismatch() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge
        let challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        
        // Create proof with correct nonce but wrong solution
        let input = format!("{}{}", challenge.challenge, "123");
        let correct_solution = issuer.pow_manager.hash_sha256(&input);
        
        let proof = PowProof {
            challenge_id: challenge.id.clone(),
            nonce: "123".to_string(),
            solution: "wrong_solution_hash".to_string(), // Wrong solution
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
        assert!(!is_valid);
        
        // Now test with correct solution
        let correct_proof = PowProof {
            challenge_id: challenge.id.clone(),
            nonce: "123".to_string(),
            solution: correct_solution,
            difficulty: "0000ffff".to_string(),
            submitted_at: Utc::now(),
            client_ip: "127.0.0.1".to_string(),
        };
        
        let is_valid = issuer.pow_manager.verify_solution(&challenge, &correct_proof).await.unwrap();
        // This should be valid if the hash meets the difficulty
        println!("Correct solution validation result: {}", is_valid);
    }
    
    #[tokio::test]
    async fn test_blake3_hashing() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        let input = "test_input_string";
        let hash = issuer.pow_manager.hash_blake3(input);
        
        // Verify it's a valid hex string
        assert!(hash.len() > 0);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Verify it's different from SHA256
        let sha256_hash = issuer.pow_manager.hash_sha256(input);
        assert_ne!(hash, sha256_hash);
    }
    
    #[tokio::test]
    async fn test_pow_token_issuance_with_valid_proof() {
        let config = Arc::new(AppConfig::default());
        let issuer = TokenIssuerAdapter::new(config);
        
        // Generate a challenge with very low difficulty
        let mut challenge = issuer.generate_pow_challenge("127.0.0.1").await.unwrap();
        challenge.target_difficulty = "ffffffff".to_string(); // Very easy difficulty
        
        // Find a valid solution
        let mut valid_proof = None;
        for nonce in 0..1000 {
            let input = format!("{}{}", challenge.challenge, nonce);
            let solution = issuer.pow_manager.hash_sha256(&input);
            
            if solution[..8] >= challenge.target_difficulty[..8] {
                let proof = PowProof {
                    challenge_id: challenge.id.clone(),
                    nonce: nonce.to_string(),
                    solution: solution.clone(),
                    difficulty: "ffffffff".to_string(),
                    submitted_at: Utc::now(),
                    client_ip: "127.0.0.1".to_string(),
                };
                
                let is_valid = issuer.pow_manager.verify_solution(&challenge, &proof).await.unwrap();
                if is_valid {
                    valid_proof = Some(proof);
                    break;
                }
            }
        }
        
        if let Some(proof) = valid_proof {
            // Create PoW token request
            let issuance_request = TokenIssuanceRequest {
                user_id: "pow_user_valid".to_string(),
                permissions: vec!["read".to_string(), "write".to_string()],
                client_ip: Some("127.0.0.1".to_string()),
                user_agent: Some("PoWTestApp/1.0".to_string()),
                custom_expiration: None,
                mode: TokenIssuanceMode::ProofOfWork(proof),
                pow_challenge: Some(challenge),
            };
            
            let result = issuer.issue_token(issuance_request).await;
            assert!(result.is_ok());
            
            let response = result.unwrap();
            assert!(response.user_id.is_some());
            assert_eq!(response.token_type, "Bearer");
            
            // Verify enhanced permissions
            let token_data = decode::<JwtClaims>(
                &response.token,
                &DecodingKey::from_secret("your-super-secret-jwt-key-that-is-at-least-32-characters-long".as_ref()),
                &Validation::new(Algorithm::HS256)
            ).unwrap();
            
            assert!(token_data.claims.permissions.contains(&"pow_validated".to_string()));
            assert!(token_data.claims.permissions.iter().any(|p| p.starts_with("rate_multiplier_")));
        }
    }

    #[tokio::test]
    async fn test_pool_token_issuance() {
        let config = Arc::new(AppConfig::default());
        let token_issuer = TokenIssuerAdapter::new(config);
        
        let share = PoolShare {
            challenge_id: "test-challenge".to_string(),
            miner_address: "test-miner".to_string(),
            nonce: "12345".to_string(),
            solution: "abcdef".to_string(),
            difficulty: 1.5,
            timestamp: Utc::now(),
            pool_signature: Some("signature".to_string()),
        };
        
        let request = TokenIssuanceRequest {
            user_id: "test-user".to_string(),
            permissions: vec!["read".to_string()],
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            custom_expiration: None,
            mode: TokenIssuanceMode::PoolValidated(share),
            pow_challenge: None,
        };
        
        // This should fail because mining pool client is not configured
        let result = token_issuer.issue_token(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mining pool client not available"));
    }

    #[tokio::test]
    async fn test_enhance_pool_permissions() {
        let config = Arc::new(AppConfig::default());
        let token_issuer = TokenIssuerAdapter::new(config);
        
        let share = PoolShare {
            challenge_id: "test-challenge".to_string(),
            miner_address: "test-miner-address".to_string(),
            nonce: "12345".to_string(),
            solution: "abcdef".to_string(),
            difficulty: 1.5,
            timestamp: Utc::now(),
            pool_signature: Some("signature".to_string()),
        };
        
        let base_permissions = vec!["read".to_string(), "write".to_string()];
        let enhanced = token_issuer.enhance_pool_permissions(&base_permissions, &share);
        
        assert!(enhanced.contains(&"pool_validated".to_string()));
        assert!(enhanced.contains(&"miner_test-miner-address".to_string()));
        assert!(enhanced.contains(&"rate_multiplier_2.0".to_string()));
        assert!(enhanced.contains(&"read".to_string()));
        assert!(enhanced.contains(&"write".to_string()));
        assert_eq!(enhanced.len(), 5); // 2 base + 3 enhanced
    }
}
