use crate::blog::{PostID, SessionID};
use crate::routes::api::SessionQuery;
use crate::state::SharedState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostOptions {
    reply_to: Option<PostID>,
}

pub(super) async fn post(
    State(state): SharedState,
    Query(query): Query<SessionQuery>,
    Json(options): Json<PostOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&query.session).await else {
        return StatusCode::UNAUTHORIZED;
    };

    let new_post_id = crate::blog::get_random_hex_string::<{ crate::blog::POST_ID_BYTES }>();
    let post_path = std::path::Path::new(crate::blog::STORE_PATH).join(&new_post_id);

    match tokio::fs::create_dir(&post_path).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Could not create folder for post {new_post_id}: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    let new_post_meta = crate::blog::Post {
        id: new_post_id,
        author_username: session.for_username,
        timestamp: chrono::Utc::now(),
        reply_to: options.reply_to,
        replies: Vec::new(),
        quotes: Vec::new(),
        in_progress: true,
    };

    match tokio::fs::write(
        post_path.join("meta.json"),
        serde_json::to_vec(&new_post_meta).expect("value should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Could not create meta {post_path:?}: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::CREATED
}
