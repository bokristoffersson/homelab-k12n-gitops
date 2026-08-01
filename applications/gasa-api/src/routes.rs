use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{auth, handlers, handlers::AppState};

pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/me", get(handlers::me))
        .route(
            "/slots",
            get(handlers::list_slots).post(handlers::create_slot),
        )
        .route("/slots/{id}", delete(handlers::delete_slot))
        .route(
            "/slots/{id}/book",
            post(handlers::book_slot).delete(handlers::cancel_booking),
        )
        .route("/calendar-url", get(handlers::calendar_url))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_proxy_auth,
        ));

    Router::new()
        .route("/health", get(handlers::health))
        .route("/ical/{token}/gasa.ics", get(handlers::ical_feed))
        .nest("/api", api)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
