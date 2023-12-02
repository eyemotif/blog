use crate::state::NestedRouter;
use axum::routing::{get, post};

mod finish;
mod image;
mod start;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/start", post(start::post))
        .route("/image/:postid", post(image::post))
        .route("/image/:postid/:filename", get(image::ws))
        .route("/finish", post(finish::post))
}
