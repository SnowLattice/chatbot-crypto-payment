use hex;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{error, info, warn};
use url::form_urlencoded;

pub fn get_user_id(init_data: &str) -> i64 {
    info!("Extracting user_id from init_data.");
    let parsed_data: HashMap<_, _> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();

    if let Some(user) = parsed_data.get("user") {
        match serde_json::from_str::<Value>(user) {
            Ok(json) => {
                if let Some(map) = json.as_object() {
                    if let Some(id) = map.get("id").and_then(|id| id.as_i64()) {
                        return id;
                    } else {
                        error!("user_id is missing or not a valid i64.");
                    }
                } else {
                    error!("JSON deserialization did not yield an object.");
                }
            }
            Err(e) => {
                error!("JSON parsing error: {}", e);
            }
        }
    } else {
        error!("Missing 'user' key in init_data.");
    }

    0
}

pub fn validate_initdata(init_data: &str, bot_token: &str) -> bool {
    if init_data.contains("query_id") && init_data.contains("hash=") {
        let parsed_data: HashMap<_, _> = form_urlencoded::parse(init_data.as_bytes())
            .into_owned()
            .collect();

        let received_hash = match parsed_data.get("hash") {
            Some(hash) => hash.clone(),
            None => {
                error!("Missing 'hash' in init_data.");
                return false;
            }
        };

        let mut data = parsed_data.clone();
        data.remove("hash");

        let mut sorted_data: Vec<_> = data.iter().collect();
        sorted_data.sort();
        let data_check_string = sorted_data
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let secret_key = match Hmac::<Sha256>::new_from_slice(b"WebAppData") {
            Ok(mut mac) => {
                mac.update(bot_token.as_bytes());
                mac.finalize().into_bytes()
            }
            Err(e) => {
                error!("Failed to initialize HMAC for secret key: {}", e);
                return false;
            }
        };

        let calculated_hash = match Hmac::<Sha256>::new_from_slice(&secret_key) {
            Ok(mut mac) => {
                mac.update(data_check_string.as_bytes());
                mac.finalize().into_bytes()
            }
            Err(e) => {
                error!("Failed to initialize HMAC for data check: {}", e);
                return false;
            }
        };

        let received_hash = match hex::decode(received_hash) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Hex decoding failed: {}", e);
                return false;
            }
        };

        if !cryptographic_hash_matches(&calculated_hash, received_hash.as_slice()) {
            warn!("Hash mismatch: provided and calculated hashes do not match.");
            return false;
        }

        info!("init_data validation succeeded.");
        true
    } else {
        error!("init_data is missing required 'query_id' or 'hash' parameters.");
        false
    }
}

fn cryptographic_hash_matches(calculated: &[u8], received: &[u8]) -> bool {
    calculated.iter().zip(received).all(|(a, b)| a == b)
}
