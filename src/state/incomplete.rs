use tokio::sync::RwLock;

use crate::blog::Post;
use crate::job::PostJob;
use std::collections::HashSet;
use std::sync::Arc;

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
        let post = Arc::new(RwLock::new(post));

        for job in [PostJob::Thumbnails, PostJob::ReplyParent] {
            if post.write().await.jobs_left.remove(&job) {
                let spawn_post = post.clone();
                let task = match job {
                    PostJob::Thumbnails => tokio::task::spawn_blocking(move || {
                        crate::job::thumbnails::run(&spawn_post.blocking_read().meta)
                    }),
                    PostJob::ReplyParent => tokio::task::spawn(async move {
                        crate::job::reply::run(&spawn_post.read().await.meta).await
                    }),
                    unr => unreachable!("{:?}", unr),
                };

                task.await.expect("task should not panic");
            }
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
