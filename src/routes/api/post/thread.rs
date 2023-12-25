use crate::blog::{Post, PostID};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

type LongestThreadFuture = dyn std::future::Future<Output = Result<Vec<Post>, StatusCode>> + Send;

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<Json<Vec<Post>>, StatusCode> {
    let thread = longest_thread(post_id).await?;

    // the previous implementation returned posts in reverse chonological order,
    // and its easier to write a clean frontend that way as well
    Ok(Json(thread.into_iter().rev().collect()))
}

async fn longest_thread(post_id: PostID) -> Result<Vec<Post>, StatusCode> {
    fn longest_thread_inner(
        parent_post: Post,
    ) -> std::pin::Pin<std::boxed::Box<LongestThreadFuture>> {
        Box::pin(async move {
            let mut child_thread_set = tokio::task::JoinSet::new();
            for child_id in parent_post.replies.clone() {
                let child_post = super::meta::get(Path(child_id)).await?.0;
                child_thread_set.spawn(longest_thread_inner(child_post));
            }

            let mut longest_child_thread: Option<Vec<Post>> = None;
            while let Some(child_thread) = child_thread_set
                .join_next()
                .await
                .transpose()
                .expect("longest_thread_inner should not panic")
                .transpose()?
            {
                longest_child_thread =
                    Some(if let Some(longest_thread) = longest_child_thread.take() {
                        if child_thread.len() > longest_thread.len() {
                            child_thread
                        } else {
                            longest_thread
                        }
                    } else {
                        child_thread
                    });
            }

            let mut posts = vec![parent_post];
            if let Some(mut longest_child_thread) = longest_child_thread {
                posts.append(&mut longest_child_thread);
            }

            Ok(posts)
        })
    }

    let post = super::meta::get(Path(post_id)).await?.0;
    let posts = longest_thread_inner(post).await?;

    Ok(posts)
}
