use crate::blog::{Post, PostID, SessionID};
use crate::job::PostJob;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = axum::extract::State<Arc<State>>;
pub type NestedRouter = axum::Router<Arc<State>>;

#[derive(Debug)]
pub struct State {
    pub sessions: RwLock<HashMap<SessionID, Session>>,
    pub posts_in_progress: RwLock<HashMap<PostID, IncompletePost>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub for_username: String,
    pub expires_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct IncompletePost {
    pub meta: Post,
    pub jobs_left: HashSet<PostJob>,
}

async fn write_post(post: &Post) {
    let post_folder_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post.id);

    match tokio::fs::write(
        post_folder_path.join("meta.json"),
        serde_json::to_vec(post).expect("post meta should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error writing meta for post {}: {err}", post.id);
        }
    }

    let user_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("user")
        .join(&post.author_username);

    let mut user = match tokio::fs::read(&user_path).await {
        Ok(it) => {
            serde_json::from_slice::<crate::blog::User>(&it).expect("user should deserialize")
        }
        Err(err) => {
            eprintln!(
                "Error reading author for user {} post {}: {err}",
                post.author_username, post.id
            );
            return;
        }
    };
    user.posts.push(post.id.clone());

    match tokio::fs::write(
        &user_path,
        serde_json::to_vec(&user).expect("user should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!(
                "Error writing updated user data for user {} post {}: {err}",
                post.author_username, post.id
            );
        }
    }
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
        let max_in_progress_post_age = chrono::Duration::minutes(30);

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

    pub async fn complete_post(&self, post: IncompletePost) {
        let post = Arc::new(RwLock::new(post));

        if post.read().await.jobs_left.contains(&PostJob::ResizeImages) {
            let spawn_post = post.clone();
            tokio::task::spawn_blocking(move || {
                crate::job::downsize_images(&spawn_post.blocking_read().meta)
            })
            .await
            .expect("task should not panic");

            post.write().await.jobs_left.remove(&PostJob::ResizeImages);
        }

        let post = Arc::into_inner(post)
            .expect("arc should have no refs")
            .into_inner();

        let new_post = Post {
            in_progress: false,
            timestamp: chrono::Utc::now(),
            ..post.meta
        };

        write_post(&new_post).await;
    }
}

impl Session {
    pub fn is_valid(&self) -> bool {
        std::time::Instant::now() < self.expires_at
    }
}
