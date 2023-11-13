use crate::state::NestedRouter;
use axum::routing::post;

mod finish;
mod image;
mod start;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/start", post(start::post))
        .route("/image", post(image::post))
        .route("/finish", post(finish::post))
}
