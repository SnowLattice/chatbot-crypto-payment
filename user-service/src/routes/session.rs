use std::sync::Arc;

use crate::controllers::session;
use crate::utils::secret::verify_signature;
use crate::ServiceState;
use axum::{
    middleware,
    routing::{get, post},
};

pub fn add_routers(
    router: axum::Router<Arc<ServiceState>>,
    state: Arc<ServiceState>,
) -> axum::Router<Arc<ServiceState>> {
    router
        .route("/api/auth/session", get(session::get_session))
        .route(
            "/api/auth/session",
            post(session::set_session).layer(middleware::from_fn_with_state(
                state.clone(),
                verify_signature,
            )),
        )
}
