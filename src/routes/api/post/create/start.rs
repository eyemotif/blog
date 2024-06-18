use crate::blog::{PostID, SessionID};
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct PostOptions {
    session: SessionID,
    #[serde(default)]
    reply_to: Option<PostID>,
    #[serde(default)]
    is_private: bool,
}

pub(super) async fn post(
    State(state): SharedState,
    Json(request): Json<PostOptions>,
) -> Result<Response, StatusCode> {
    let Some(session) = state.get_session(&request.session).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let user = crate::routes::api::user::get(axum::extract::Path(session.for_username))
        .await?
        .0;
    if !user.permissions.can_create_posts {
        return Err(StatusCode::FORBIDDEN);
    }

    let new_post_id = crate::blog::get_random_hex_string::<{ crate::blog::POST_ID_BYTES }>();
    let post_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&new_post_id);

    let initial_post_jobs = get_initial_post_jobs(&request);
    let new_post_meta = crate::blog::Post {
        id: new_post_id.clone(),
        author_username: user.username,
        timestamp: chrono::Utc::now(),
        reply_to: request.reply_to,
        replies: Vec::new(),
        quotes: Vec::new(),
        in_progress: true,
        is_private: request.is_private, // TODO: add separate endpoint for setting `post.private`
    };

    state.posts_in_progress.write().await.insert(
        new_post_id.clone(),
        crate::state::incomplete::IncompletePost {
            meta: new_post_meta.clone(),
            jobs_left: initial_post_jobs.iter().copied().collect(),
            media: crate::state::incomplete::Media::default(),
        },
    );

    tokio::spawn(async move { state.cleanup_stale_posts().await });

    match tokio::fs::create_dir(&post_path).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Could not create folder for post {new_post_id}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match tokio::fs::write(
        post_path.join("meta.json"),
        serde_json::to_vec(&new_post_meta).expect("value should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Could not create meta {post_path:?}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok((StatusCode::CREATED, new_post_id).into_response())
}

fn get_initial_post_jobs(request: &PostOptions) -> &'static [crate::job::PostJob] {
    if request.reply_to.is_some() {
        &[
            crate::job::PostJob::AddText,
            crate::job::PostJob::ReplyParent,
        ]
    } else {
        &[crate::job::PostJob::AddText]
    }
}
