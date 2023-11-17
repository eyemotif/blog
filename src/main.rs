use axum::ServiceExt;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

mod auth;
mod blog;
mod job;
mod joinqueue;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    let state = std::sync::Arc::new(state::State::new());
    reprocess_posts(state.clone())
        .await
        .expect("error reprocessing in-progress posts");

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

async fn reprocess_posts(state: std::sync::Arc<crate::state::State>) -> std::io::Result<()> {
    let posts_folder = std::path::Path::new(crate::blog::STORE_PATH).join("post");
    let mut dir = tokio::fs::read_dir(&posts_folder).await?;

    while let Some(entry) = dir.next_entry().await? {
        let meta = tokio::fs::read(entry.path().join("meta.json")).await?;
        let meta = serde_json::from_slice::<crate::blog::Post>(&meta)
            .expect("post metadata should deserialize");

        if meta.in_progress {
            println!("Found in-progress post: {}", meta.id);

            state
                .complete_post(crate::state::IncompletePost {
                    meta: meta.clone(),
                    jobs_left: crate::job::PostJob::all_processing_jobs(),
                })
                .await;

            println!("Post {} complete!", meta.id);
        }
    }

    Ok(())
}
