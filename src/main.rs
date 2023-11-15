use axum::ServiceExt;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

mod auth;
mod blog;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    let state = std::sync::Arc::new(state::State::new());

    // TODO: proper CORS setup
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::exact(
            axum::http::HeaderValue::from_static("https://frith.gay"),
        ))
        .allow_headers(tower_http::cors::Any);

    let app = NormalizePathLayer::trim_trailing_slash().layer(
        axum::Router::new()
            .nest("/api", routes::api::route())
            .with_state(state)
            .layer(cors),
    );

    axum::Server::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], 8010)))
        .serve(app.into_make_service())
        .await
        .expect("Error serving app")
}
