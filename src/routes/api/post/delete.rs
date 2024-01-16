use crate::blog::{PostID, SessionID};
use crate::state::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct DeleteOptions {
    session: SessionID,
}

pub(super) async fn post(
    State(state): SharedState,
    Path(post_id): Path<PostID>,
    Json(request): Json<DeleteOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&request.session).await else {
        return StatusCode::UNAUTHORIZED;
    };
    let post = match super::meta::get(Path(post_id.clone())).await {
        Ok(it) => it.0,
        Err(err) => return err,
    };

    if post.author_username != session.for_username {
        return StatusCode::FORBIDDEN;
    }

    match tokio::fs::remove_dir_all(
        std::path::Path::new(crate::blog::STORE_PATH)
            .join("post")
            .join(&post_id),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error deleting files for post {post_id}: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    let mut user = match crate::routes::api::user::get(Path(session.for_username.clone())).await {
        Ok(it) => it.0,
        Err(err) => return err,
    };
    user.posts.retain(|id| *id != post_id);

    match tokio::fs::write(
        std::path::Path::new(crate::blog::STORE_PATH)
            .join("user")
            .join(format!("{}.json", session.for_username)),
        serde_json::to_vec(&user).expect("user should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error updating user for deleted post {post_id}: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    state.posts_in_progress.write().await.remove(&post_id);
    state.cache.write().await.latest_posts = None; // HACK: invalidates the whole cache when a change is made

    StatusCode::OK
}
