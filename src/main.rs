use axum::ServiceExt;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;

mod state;

#[tokio::main]
async fn main() {
    let app = NormalizePathLayer::trim_trailing_slash().layer(axum::Router::new());

    axum::Server::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], 8010)))
        .serve(app.into_make_service())
        .await
        .expect("Error serving app")
}
