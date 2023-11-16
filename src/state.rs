use crate::blog::{Post, PostID, SessionID};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = axum::extract::State<Arc<State>>;
pub type NestedRouter = axum::Router<Arc<State>>;

#[derive(Debug)]
pub struct State {
    pub sessions: RwLock<HashMap<SessionID, Session>>,
    pub posts_in_progress: RwLock<HashMap<PostID, Post>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub for_username: String,
    pub expires_at: std::time::Instant,
}

impl State {
    pub fn new() -> State {
        State {
            sessions: RwLock::new(HashMap::new()),
            posts_in_progress: RwLock::new(HashMap::new()),
        }
    }
    pub async fn get_session(&self, session_id: &SessionID) -> Option<Session> {
        let sessions = self.sessions.read().await;
        let Some(session) = sessions.get(session_id) else {
            // TODO: cleanup expired sessions
            return None;
        };

        session.is_valid().then(|| session.clone())
    }
    pub async fn create_session(
        &self,
        for_username: String,
        _auth: crate::auth::Auth,
    ) -> SessionID {
        let session_id: SessionID =
            crate::blog::get_random_hex_string::<{ crate::blog::SESSION_ID_BYTES }>();
        let new_session = Session {
            for_username,
            expires_at: std::time::Instant::now() + crate::blog::SESSION_EXPIRED_AFTER,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), new_session);

        session_id
    }

    pub async fn cleanup_stale_posts(&self) {
        let max_in_progress_post_age = chrono::Duration::days(1);

        let now = chrono::Utc::now();
        let mut posts = self.posts_in_progress.write().await;

        let stale_post_ids = posts
            .iter()
            .filter_map(|(id, post)| {
                let elapsed = now - post.timestamp;
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

impl Session {
    pub fn is_valid(&self) -> bool {
        std::time::Instant::now() < self.expires_at
    }
}
