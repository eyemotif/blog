use crate::blog::{Post, STORE_PATH};
use crate::state::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

pub(super) async fn get(
    State(state): SharedState,
    region: Option<Path<(usize, usize)>>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    let Path((amount, after)) = region.unwrap_or(Path((10, 0)));

    let cache_len =
        if let Some(lastest_posts_cache) = state.cache.read().await.latest_posts.as_ref() {
            lastest_posts_cache.len()
        } else {
            0
        };

    if cache_len >= amount + after {
        Ok(Json(
            state
                .cache
                .read()
                .await
                .latest_posts
                .as_ref()
                .expect("cache_len should be greater than zero")
                .iter()
                .skip(after)
                .take(amount)
                .cloned()
                .collect(),
        ))
    } else {
        let latest_posts = get_latest_posts(amount + after).await?;
        state.cache.write().await.latest_posts = Some(latest_posts.clone());

        Ok(Json(
            latest_posts.into_iter().skip(after).take(amount).collect(),
        ))
    }
}

async fn get_latest_posts(amount: usize) -> Result<Vec<Post>, StatusCode> {
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

    let mut latest_post_ids = Vec::new();

    for user in users {
        if user.posts.is_empty() {
            continue;
        }

        // posts are in reverse chronological order
        latest_post_ids.extend_from_slice(&user.posts[user.posts.len().saturating_sub(amount)..]);
    }

    let mut latest_posts = Vec::new();
    for post in latest_post_ids {
        let post = super::meta::get(Path(post)).await?.0;
        if post.in_progress {
            continue;
        }

        latest_posts.push(post);
    }

    latest_posts.sort_unstable_by(|a, b| a.timestamp.cmp(&b.timestamp).reverse());

    Ok(latest_posts)
}
