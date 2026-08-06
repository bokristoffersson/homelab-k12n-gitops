use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{auth, handlers, handlers::AppState, korschema};

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
        .route("/korschema/schedule", get(korschema::schedule))
        .route("/korschema/students", get(korschema::students))
        .route("/korschema/progress/{student}", get(korschema::progress))
        .route(
            "/korschema/progress/{student}/checks/{exercise_id}",
            put(korschema::set_check).delete(korschema::clear_check),
        )
        .route(
            "/korschema/progress/{student}/notes/{lesson_id}",
            post(korschema::add_note),
        )
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
