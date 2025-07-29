use serde::Serialize;
#[derive(Debug, Clone, Default, Serialize)]
pub struct UserResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSessionResponse {
    pub subscription_status: bool,
    pub credits_remaining: i64,
    pub preferences: serde_json::Value,
    pub session_metadata: serde_json::Value,
}
