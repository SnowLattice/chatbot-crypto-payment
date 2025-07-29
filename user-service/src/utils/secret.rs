use std::sync::Arc;

use crate::ServiceState;
use axum::extract::State;
use axum::{body, extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::error;
type HmacSha256 = Hmac<Sha256>;

pub async fn verify_signature(
    State(state): State<Arc<ServiceState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let signature = parts.headers.get("X-Signature").ok_or_else(|| {
        let error_message =
            "Missing 'X-Signature' header: signature verification failed".to_string();
        error!("{}", error_message);
        (StatusCode::UNAUTHORIZED, error_message)
    })?;
    let signature_str = signature.to_str().map_err(|_| {
        let error_message =
            "Malformed 'X-Signature' header: expected valid UTF-8 string".to_string();
        error!("{}", error_message);
        (StatusCode::BAD_REQUEST, error_message)
    })?;

    let signature_bytes = BASE64_STANDARD.decode(signature_str).map_err(|_| {
        let error_message =
            "Signature decoding failed: 'X-Signature' header contains invalid Base64 string"
                .to_string();
        error!("{}", error_message);
        (StatusCode::BAD_REQUEST, error_message)
    })?;

    let whole_body = body::to_bytes(body, usize::MAX).await.map_err(|_| {
        let error_message =
            "Request body reading failed: unable to read the body content".to_string();
        error!("{}", error_message);
        (StatusCode::BAD_REQUEST, error_message)
    })?;
    let mut mac = HmacSha256::new_from_slice(state.config.secret.inter_secret_key.as_bytes())
        .map_err(|_| {
            let error_message =
                "HMAC initialization error: provided secret key is invalid".to_string();
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })?;
    mac.update(&whole_body);

    mac.verify_slice(&signature_bytes).map_err(|_| {
        let error_message =
            "Invalid signature: signature verification against the request body failed".to_string();
        error!("{}", error_message);
        (StatusCode::UNAUTHORIZED, error_message)
    })?;

    Ok(next
        .run(Request::from_parts(parts, whole_body.into()))
        .await)
}
