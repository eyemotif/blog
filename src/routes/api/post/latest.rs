use crate::blog::{Post, STORE_PATH};
use crate::state::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

// TODO: pagination and/or different lengths
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
    let mut latest_posts: Vec<Post> = Vec::with_capacity(amount);

    'users: for user in get_all_users().await? {
        // user posts are stored in chronological order
        for post in user.posts.into_iter().rev() {
            let post = super::meta::get(Path(post)).await?.0;
            if post.in_progress || post.is_reply() {
                continue;
            }

            if let Some(last_post) = latest_posts.last() {
                if post.timestamp < last_post.timestamp {
                    if latest_posts.len() < amount {
                        latest_posts.push(post);
                        continue;
                    }
                    continue 'users;
                }
            } else {
                latest_posts.push(post);
                continue;
            }

            for cursor in (0..latest_posts.len()).rev() {
                if post.timestamp < latest_posts[cursor].timestamp {
                    latest_posts.insert(cursor, post);
                    break;
                }
                if cursor == 0 {
                    latest_posts.insert(0, post);
                    break; // redundant break but the compiler isnt smart enough to tell (yet!)
                }
            }

            if latest_posts.len() > amount {
                latest_posts.pop();
            }
        }
    }

    Ok(latest_posts)
}

async fn get_all_users() -> Result<Vec<crate::blog::User>, StatusCode> {
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

    Ok(users)
}
