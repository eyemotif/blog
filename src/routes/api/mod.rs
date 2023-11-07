use crate::state::NestedRouter;
use axum::routing::{get, post};

mod post;
mod session;
mod user;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .nest("/post", post::route())
        .route("/user/:id", get(user::get))
        .route("/session", post(session::post))
}
