use crate::state::NestedRouter;
use axum::routing::post;

mod finish;
mod image;
mod start;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/start", post(start::post))
        .route(
            "/image/:postid/:filename",
            post(image::post).on(axum::routing::MethodFilter::all(), image::ws),
        )
        .route("/finish", post(finish::post))
}
