use std::sync::Arc;

use crate::controllers::crypto;
use crate::ServiceState;
use axum::routing::post;

pub fn add_routers(router: axum::Router<Arc<ServiceState>>) -> axum::Router<Arc<ServiceState>> {
    router
        .route("/api/payment/crypto", post(crypto::subscribe))
        .route(
            "/api/payment/webhook/:user_id",
            post(crypto::callback_webhook),
        )
}
