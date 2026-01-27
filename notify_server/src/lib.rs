use axum:: {
    routing::get,
    Router,
};
mod sse;
use crate::sse::sse_handler;


pub fn get_router()-> Router {
    Router::new().route("/events", get(sse_handler))
}