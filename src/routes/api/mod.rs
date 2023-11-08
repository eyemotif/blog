use crate::blog::SessionID;
use crate::state::NestedRouter;
use axum::routing::{get, post};
use serde::Deserialize;

mod post;
mod session;
mod user;

#[derive(Debug, Deserialize)]
struct SessionQuery {
    pub session: SessionID,
}

pub fn route() -> NestedRouter {
    axum::Router::new()
        .nest("/post", post::route())
        .route("/user/:id", get(user::get))
        .route("/session", post(session::post))
}
