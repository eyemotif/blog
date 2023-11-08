use crate::blog::{Post, PostID};
use crate::routes::api::SessionQuery;
use crate::state::SharedState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostFinishOptions {
    post_id: PostID,
    text: String,
}

pub(super) async fn put(
    State(state): SharedState,
    Query(query): Query<SessionQuery>,
    Json(options): Json<PostFinishOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&query.session).await else {
        return StatusCode::UNAUTHORIZED;
    };

    let post =
        match crate::routes::api::post::meta::get(axum::extract::Path(options.post_id.clone()))
            .await
        {
            Ok(Json(it)) => it,
            Err(err) => return err,
        };
    if post.author_username != session.for_username {
        return StatusCode::FORBIDDEN;
    }

    let new_post = Post {
        in_progress: true,
        ..post
    };

    let post_folder_path = std::path::Path::new(crate::blog::STORE_PATH).join(&options.post_id);

    match tokio::fs::write(post_folder_path.join("text.md"), options.text).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error writing text for post {}: {err}", options.post_id);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    match tokio::fs::write(
        post_folder_path.join("meta.json"),
        serde_json::to_vec(&new_post).expect("post meta should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error writing meta for post {}: {err}", options.post_id);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}
