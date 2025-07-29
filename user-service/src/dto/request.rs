use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SetSessionRequest {
    pub user_id: i64,
    pub subscription_status: Option<bool>,
    pub credits_remaining: Option<i64>,
    pub preferences: Option<serde_json::Value>,
    pub session_metadata: Option<serde_json::Value>,
}
