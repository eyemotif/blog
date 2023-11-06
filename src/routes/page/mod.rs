use crate::state::NestedRouter;
use axum::routing::get;

mod home;
mod post;
mod user;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/", get(home::get))
        .route("/post/:id", get(post::get))
        .route("/user/:id", get(user::get))
}
