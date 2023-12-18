use crate::blog::{PostID, SessionID};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod incomplete;
pub mod session;

pub type SharedState = axum::extract::State<Arc<State>>;
pub type NestedRouter = axum::Router<Arc<State>>;

#[derive(Debug)]
pub struct State {
    pub sessions: RwLock<HashMap<SessionID, session::Session>>,
    pub posts_in_progress: RwLock<HashMap<PostID, incomplete::IncompletePost>>,
}

impl State {
    pub fn new() -> State {
        State {
            sessions: RwLock::new(HashMap::new()),
            posts_in_progress: RwLock::new(HashMap::new()),
        }
    }

    pub async fn cleanup_stale_posts(&self) {
        let max_in_progress_post_age = chrono::Duration::from_std(crate::blog::INCOMPLETE_POST_TTL)
            .expect("Constant std::duration be in range of chrono::duration");
        let now = chrono::Utc::now();
        let mut posts = self.posts_in_progress.write().await;

        let stale_post_ids = posts
            .iter()
            .filter_map(|(id, post)| {
                let elapsed = now - post.meta.timestamp;
                (elapsed >= max_in_progress_post_age).then(|| id.clone())
            })
            .collect::<Vec<_>>();

        if !stale_post_ids.is_empty() {
            println!(
                "found {} stale in-progress posts, deleting them now",
                stale_post_ids.len()
            );
        }

        for stale_post_id in stale_post_ids {
            match tokio::fs::remove_dir_all(
                std::path::Path::new(crate::blog::STORE_PATH)
                    .join("post")
                    .join(&stale_post_id),
            )
            .await
            {
                Ok(()) => {
                    posts.remove(&stale_post_id);
                }
                Err(err) => eprintln!("Error cleaning up stale post {stale_post_id}: {err}"),
            }
        }
    }
}
