use crate::state::NestedRouter;
use axum::routing::get;

mod post;
mod user;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .nest("/post", post::route())
        .route("/user/:id", get(user::get))
}
