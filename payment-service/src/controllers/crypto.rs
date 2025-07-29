use crate::utils::subscription::SubscriptionKey;
use crate::{
    dto::response::CryptoSubscriptionResponse, utils::jwt::UserClaims,
    utils::session::send_session_data, utils::subscription, ServiceState,
};
use axum::extract::Path;
use axum::{
    body::Bytes,
    extract::State,
    http::header::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use core::str;
use hmac::{Hmac, Mac};
use reqwest;
use serde_json::{json, Value};
use sha2::Sha512;
use std::sync::Arc;
use tracing::{error, info};

type HmacSha512 = Hmac<Sha512>;

fn format_error(message: &str, error: impl std::fmt::Display) -> (StatusCode, String) {
    let error_message = format!("{}: {}", message, error);
    error!("Error occurred: {}", error_message);
    (StatusCode::INTERNAL_SERVER_ERROR, error_message)
}

pub async fn subscribe(
    State(state): State<Arc<ServiceState>>,
    user: UserClaims,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // let subscription_key = subscription::SubscriptionKey { user_id: user.uid };
    // if let Some(url) = subscription::get(&state.redis, &subscription_key)
    //     .await
    //     .map_err(|e| {
    //         let error_message = format!("Redis error: {}", e);
    //         error!("Redis error: {}", error_message);
    //         (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    //     })?
    // {
    //     return Ok(Json(CryptoSubscriptionResponse { invoice_url: url }).into_response());
    // }
    let invoice_request = json!({
        "merchant": state.config.oxapay.merchant_key,
        "amount": 1,
        "currency" : "TRX",
        "lifeTime" : 15,
        "returnUrl": state.config.oxapay.return_url,
        "callbackUrl": format!("{}/api/payment/webhook/{}", state.config.oxapay.callback_urlbase, user.uid)
    });
    info!(
        "{}",
        format!(
            "{}/api/payment/webhook/{}",
            state.config.oxapay.callback_urlbase, user.uid
        )
    );
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.oxapay.com/merchants/request")
        .json(&invoice_request)
        .send()
        .await
        .map_err(|e| format_error("Failed to send invoie creation requset", e))?;

    match response.status().is_success() {
        true => {
            let response_body = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| format_error("Failed to parse Oxapay invoice response in json", e))?;
            if let Some(result) = response_body.get("result") {
                match result.as_i64() {
                    Some(100) => {
                        if let Some(pay_link) = response_body.get("payLink") {
                            if let Some(pay_str) = pay_link.as_str() {
                                let key = SubscriptionKey { user_id: user.uid };
                                subscription::set(&state.redis, (&key, &pay_str.to_string()))
                                    .await
                                    .map_err(|e| {
                                        format_error("Failed to save pay link into redis", e)
                                    })?;
                                info!("Subscription payment link({}) is successfully generated of the user {}", pay_str, user.uid);
                                return Ok(Json(CryptoSubscriptionResponse {
                                    invoice_url: pay_str.to_string(),
                                })
                                .into_response());
                            } else {
                                return Err(format_error(
                                    "Failed to parse Oxapay invoice response in JSON",
                                    "payLink field is not a string",
                                ));
                            }
                        } else {
                            return Err(format_error(
                                "Failed to parse Oxapay invoice response in json",
                                "payLink field is missing",
                            ));
                        }
                    }
                    _ => {
                        return Err(format_error(
                            "Failed to send Oxapay invoice request",
                            response_body
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("message field is missing"),
                        ));
                    }
                }
            } else {
                return Err(format_error(
                    "Failed to parse Oxapay invoice response in json",
                    "result field is missing",
                ));
            }
        }
        _ => {
            return Err(format_error(
                "Failed to send Oxapay invoice request",
                "bad request",
            ));
        }
    }
}

pub async fn callback_webhook(
    Path(user_id): Path<i64>,
    State(state): State<Arc<ServiceState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(hmac_header) = headers.get("hmac") {
        let body_str = str::from_utf8(&body).unwrap_or("{}");
        let payload: Value = serde_json::from_str(body_str).unwrap_or_default();

        if let Some(payment_type) = payload.get("type").and_then(|t| t.as_str()) {
            if payment_type == "payment" {
                let merchant_key = state.config.oxapay.merchant_key.clone();
                let mut mac =
                    HmacSha512::new_from_slice(merchant_key.as_bytes()).map_err(|_| {
                        error!("Invalid HMAC key configuration");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            String::from("Webhook configuration error"),
                        )
                    })?;
                mac.update(&body);
                let hmac_str = hmac_header.to_str().map_err(|e| {
                    error!("HMAC header parsing failed: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        String::from("HMAC header parsing failed"),
                    )
                })?;

                let decoded_hmac = hex::decode(hmac_str).map_err(|e| {
                    error!("Failed to decode HMAC header: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        String::from("Failed to decode HMAC header"),
                    )
                })?;
                mac.verify_slice(&decoded_hmac).map_err(|_| {
                    error!("HMAC verification failed");
                    (
                        StatusCode::UNAUTHORIZED,
                        String::from("HMAC verification failed"),
                    )
                })?;

                

                if let Some(status) = payload.get("status").and_then(|t| t.as_str()) {
                    info!("Payload verification successful for user {} with the status of {}", user_id, status);
                    if status == "Paid" {
                        send_session_data(
                            json!({
                                "user_id" : user_id,
                                "subscription_status" : true,
                            }),
                            state.config.server.auth_service.clone(),
                            state.config.server.auth_secret_key.clone(),
                        )
                        .await
                        .map_err(|e| {
                            error!("{}", format!("Failed to send updated session data : {}", e));
                            (
                                StatusCode::BAD_REQUEST,
                                format!("Failed to send updated session data : {}", e),
                            )
                        })?;
                    }

                    return Ok("Webhook success".into_response());
                }
            }
        }
    }
    error!("Invalid webhook request: missing or incorrect HMAC");
    Err((StatusCode::BAD_REQUEST, String::from("Webhook failed")))
}