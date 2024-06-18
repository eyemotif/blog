use tokio::sync::RwLock;

use crate::blog::Post;
use crate::job::PostJob;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct IncompletePost {
    pub meta: Post,
    pub jobs_left: HashSet<PostJob>,
    pub media: Media,
}

#[derive(Debug, Clone, Default)]
pub struct Media {
    pub images: Vec<String>,
    pub videos: Vec<String>,
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
        .join(format!("{}.json", post.author_username));

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

impl super::State {
    pub async fn complete_post(&self, post: IncompletePost) {
        let jobs_left = post.jobs_left.clone();
        let post = Arc::new(RwLock::new(post));

        let mut set = tokio::task::JoinSet::new();
        for job in jobs_left {
            let spawn_post = post.clone();
            match job {
                PostJob::Thumbnails => set.spawn_blocking(move || {
                    crate::job::thumbnails::run(&spawn_post.blocking_read());
                }),
                PostJob::ReplyParent => set.spawn(async move {
                    crate::job::reply::run(&spawn_post.read().await.meta).await;
                }),
                PostJob::AddText => unreachable!(),
            };
        }

        while let Some(task_result) = set.join_next().await {
            task_result.expect("task should not panic");
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

        // HACK: invalidates the whole cache when a change is made
        self.cache.write().await.latest_posts = None;
    }
}
