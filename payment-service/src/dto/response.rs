use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionData {
    pub credits_remaining: f64,
    pub preferences: serde_json::Value,
    pub session_metadata: serde_json::Value,
    pub subscription_status: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoSubscriptionResponse {
    pub invoice_url: String,
}
