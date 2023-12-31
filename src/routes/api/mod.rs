use crate::state::NestedRouter;
use axum::routing::{get, post};

mod invite;
mod member;
mod post;
mod session;
mod signup;
mod user;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .nest("/post", post::route())
        .nest("/member", member::route())
        .route("/user/:id", get(user::get))
        .route("/session", post(session::post))
        .route("/invite", post(invite::post))
        .route("/signup", post(signup::post))
}
