use crate::blog::{PostID, SessionID};
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct PostFinishOptions {
    session: SessionID,
    post_id: PostID,
    text: String,
}

pub(super) async fn post(
    State(state): SharedState,
    Json(request): Json<PostFinishOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&request.session).await else {
        return StatusCode::UNAUTHORIZED;
    };

    let Some(mut post) = state
        .posts_in_progress
        .write()
        .await
        .remove(&request.post_id)
    else {
        return StatusCode::NOT_FOUND;
    };

    if !post.meta.in_progress {
        eprintln!("Completed post {} in in-progress post list!", post.meta.id);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if post.meta.author_username != session.for_username {
        return StatusCode::FORBIDDEN;
    }
    if request.text.trim().is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    match tokio::fs::write(
        std::path::Path::new(crate::blog::STORE_PATH)
            .join("post")
            .join(&post.meta.id)
            .join("text.md"),
        request.text,
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error writing text for post {}: {err}", request.post_id);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    post.jobs_left.remove(&crate::job::PostJob::AddText);
    tokio::task::spawn(async move { state.complete_post(post).await });

    StatusCode::OK
}
