use crate::state::NestedRouter;
use axum::routing::{get, post};

mod create;
mod delete;
mod image;
mod latest;
mod meta;
mod text;
mod thread;

pub fn route() -> NestedRouter {
    let image_compression_layer = tower_http::compression::CompressionLayer::new()
        .br(true)
        .quality(tower_http::CompressionLevel::Best);

    axum::Router::new()
        .route("/:id/meta", get(meta::get))
        .route("/:id/text", get(text::get))
        .route("/:id/text/member", get(text::get_with_session))
        .route("/latest/:amount/:after", get(latest::get))
        .route(
            "/:id/image/:img",
            get(image::get).layer(image_compression_layer),
        )
        .route("/:id/delete", post(delete::post))
        .route("/thread/:id", get(thread::get))
        .nest("/create", create::route())
}
