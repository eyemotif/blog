use axum::ServiceExt;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;

mod auth;
mod blog;
mod job;
// mod joinqueue;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    let state = std::sync::Arc::new(state::State::new());
    restore_incomplete_posts(state.clone())
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

async fn restore_incomplete_posts(
    state: std::sync::Arc<crate::state::State>,
) -> std::io::Result<()> {
    async fn try_restore_post(
        path: std::path::PathBuf,
        state: std::sync::Arc<crate::state::State>,
    ) -> std::io::Result<Option<crate::blog::PostID>> {
        let meta = tokio::fs::read(path.join("meta.json")).await?;
        let meta = serde_json::from_slice::<crate::blog::Post>(&meta)
            .expect("post metadata should deserialize");

        if !meta.in_progress {
            return Ok(None);
        }

        let post_id = meta.id.clone();
        state.posts_in_progress.write().await.insert(
            post_id.clone(),
            state::incomplete::IncompletePost {
                jobs_left: crate::job::PostJob::all_possible_processing_jobs(&meta),
                meta,
            },
        );

        Ok(Some(post_id))
    }

    let mut posts_dir =
        tokio::fs::read_dir(std::path::Path::new(crate::blog::STORE_PATH).join("post")).await?;
    let mut process_set = tokio::task::JoinSet::new();

    while let Some(entry) = posts_dir.next_entry().await? {
        process_set.spawn(try_restore_post(entry.path(), state.clone()));
    }

    while let Some(maybe_restored_post_id) = process_set
        .join_next()
        .await
        .transpose()
        .expect("task should not panic")
        .transpose()?
    {
        let Some(restored_post_id) = maybe_restored_post_id else {
            continue;
        };

        println!("Restored incomplete post {restored_post_id}");
    }

    state.cleanup_stale_posts().await;

    Ok(())
}
