use crate::state::NestedRouter;
use axum::routing::get;

mod create;
mod latest;
mod meta;
mod text;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/meta/:id", get(meta::get))
        .route("/text/:id", get(text::get))
        .route("/latest/:amount/:after", get(latest::get))
        .nest("/create", create::route())
}
