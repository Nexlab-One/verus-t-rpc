//! Payments domain models and types

use serde::{Deserialize, Serialize};

/// Supported shielded address types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShieldedAddressType {
    Orchard,
    Sapling,
}

impl ShieldedAddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShieldedAddressType::Orchard => "orchard",
            ShieldedAddressType::Sapling => "sapling",
        }
    }
}

impl std::str::FromStr for ShieldedAddressType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "orchard" => Ok(ShieldedAddressType::Orchard),
            "sapling" => Ok(ShieldedAddressType::Sapling),
            _ => Err(format!("unsupported address type: {}", s)),
        }
    }
}

/// Payment tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTier {
    pub id: String,
    pub amount_vrsc: f64,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// Payment session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Submitted,
    Verified,
    Confirmed1,
    Finalized,
    Failed,
    Expired,
}

/// Payment session persisted in the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSession {
    pub payment_id: String,
    pub tier_id: String,
    pub address: String,
    pub address_type: ShieldedAddressType,
    pub amount_vrsc: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub status: PaymentStatus,
    pub txid: Option<String>,
    pub confirmations: u32,
    pub provisional_token: Option<String>,
    pub final_token: Option<String>,
}

impl PaymentSession {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}


