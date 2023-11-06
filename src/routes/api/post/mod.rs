use crate::state::NestedRouter;
use axum::routing::get;

mod meta;
mod text;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/meta/:id", get(meta::get))
        .route("/text/:id", get(text::get))
}
