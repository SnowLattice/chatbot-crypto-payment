use std::sync::Arc;

use crate::controllers::user;
use crate::ServiceState;
use axum::routing::post;

pub fn add_routers(router: axum::Router<Arc<ServiceState>>) -> axum::Router<Arc<ServiceState>> {
    router
        .route("/api/auth/login", post(user::login))
        .route("/api/auth/refresh", post(user::refresh))
}
