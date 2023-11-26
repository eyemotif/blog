use crate::state::NestedRouter;
use axum::routing::get;

mod create;
mod image;
mod latest;
mod meta;
mod text;

pub fn route() -> NestedRouter {
    let image_compression_layer = tower_http::compression::CompressionLayer::new()
        .br(true)
        .quality(tower_http::CompressionLevel::Best);

    // TODO: /:id/..
    axum::Router::new()
        .route("/:id/meta/", get(meta::get))
        .route("/:id/text/", get(text::get))
        .route("/latest/:amount/:after", get(latest::get))
        .route(
            "/:id/image/:img",
            get(image::get).layer(image_compression_layer),
        )
        .nest("/create", create::route())
}
