pub mod session;
pub mod user;
use std::sync::Arc;

use crate::ServiceState;
use axum::Router;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
pub fn create_router(state: Arc<ServiceState>) -> Router {
    let router = Router::new();
    let router = user::add_routers(router);
    let router = session::add_routers(router, state.clone());

    router.with_state(state).layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    )
}
