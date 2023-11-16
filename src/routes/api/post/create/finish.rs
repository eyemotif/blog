use crate::blog::{Post, PostID, STORE_PATH};
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

pub(super) async fn post(
    State(state): SharedState,
    Query(query): Query<SessionQuery>,
    Json(options): Json<PostFinishOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&query.session).await else {
        return StatusCode::UNAUTHORIZED;
    };

    let mut posts_in_progress = state.posts_in_progress.write().await;
    let Some(post) = posts_in_progress.get(&options.post_id).cloned() else {
        return StatusCode::NOT_FOUND;
    };

    if !post.in_progress {
        return StatusCode::CONFLICT;
    }
    if post.author_username != session.for_username {
        return StatusCode::FORBIDDEN;
    }
    if options.text.trim().is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let new_post = Post {
        in_progress: false,
        timestamp: chrono::Utc::now(),
        ..post
    };

    let post_folder_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&options.post_id);

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

    let mut user = match crate::routes::api::user::get(axum::extract::Path(
        session.for_username.clone(),
    ))
    .await
    {
        Ok(Json(it)) => it,
        Err(err) => return err,
    };
    user.posts.push(options.post_id.clone());

    match tokio::fs::write(
        std::path::Path::new(STORE_PATH)
            .join("user")
            .join(format!("{}.json", session.for_username)),
        serde_json::to_vec(&user).expect("user should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!(
                "Error writing updated user data for user {} post {}: {err}",
                options.post_id, session.for_username
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    posts_in_progress.remove(&options.post_id);

    StatusCode::OK
}
