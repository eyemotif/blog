use crate::blog::{Post, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

pub(super) async fn get(
    region: Option<Path<(usize, usize)>>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    let mut users_files =
        match tokio::fs::read_dir(std::path::Path::new(STORE_PATH).join("user")).await {
            Ok(it) => it,
            Err(err) => {
                eprintln!("Error reading users folder: {err}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    let mut users = Vec::<crate::blog::User>::new();

    while let Some(user) = users_files.next_entry().await.transpose() {
        let user = match user {
            Ok(it) => it,
            Err(err) => {
                eprintln!("Error reading file in users folder: {err}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let user_path = user.path();

        let user = match tokio::fs::read(user.path()).await {
            Ok(it) => it,
            Err(err) => {
                eprintln!("Error reading file {user_path:?} in users folder: {err}",);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        match serde_json::from_slice(&user) {
            Ok(user) => users.push(user),
            Err(err) => {
                eprintln!("Error reading file {user_path:?} in users folder: {err}",);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let Path((amount, after)) = region.unwrap_or(Path((10, 0)));
    let posts_to_read = amount + after;
    let mut latest_post_ids = Vec::new();

    for user in users {
        if user.posts.is_empty() {
            continue;
        }

        // posts are in reverse chronological order
        latest_post_ids
            .extend_from_slice(&user.posts[user.posts.len().saturating_sub(posts_to_read)..]);
    }

    if latest_post_ids.len() <= after {
        return Ok(Json(Vec::new()));
    }

    let mut latest_posts = Vec::new();
    for post in latest_post_ids {
        let post = super::meta::get(Path(post)).await?.0;
        if post.in_progress {
            continue;
        }

        latest_posts.push(post);
    }

    // TODO: is this sorted correctly?
    latest_posts.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp).reverse());
    let latest_posts = latest_posts.into_iter().skip(after).take(amount).collect();

    Ok(Json(latest_posts))
}
