use crate::blog::{Post, PostID};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

type LongestThreadFuture =
    dyn std::future::Future<Output = Result<(usize, Post), StatusCode>> + Send;

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<Json<Vec<Post>>, StatusCode> {
    let (_, last_child) = longest_thread(post_id.clone()).await?;

    let mut thread = vec![last_child];

    loop {
        let earliest_post = thread.last().unwrap();
        if earliest_post.id == post_id {
            break;
        }

        let Some(parent) = earliest_post.reply_to.as_ref() else {
            break;
        };
        let parent = super::meta::get(Path(parent.clone())).await?.0;
        thread.push(parent);
    }

    Ok(Json(thread))
}

// TODO: pass `Post` vectors up so `get` doesnt have to query them again
fn longest_thread(post_id: PostID) -> std::pin::Pin<std::boxed::Box<LongestThreadFuture>> {
    Box::pin(async move {
        let post = super::meta::get(Path(post_id)).await?.0;

        let mut set = tokio::task::JoinSet::new();
        for child_id in post.replies.clone() {
            set.spawn(longest_thread(child_id));
        }

        let mut counts = Vec::new();
        while let Some(count) = set
            .join_next()
            .await
            .transpose()
            .expect("count_children should not panic")
            .transpose()?
        {
            counts.push(count);
        }

        Ok(counts
            .into_iter()
            .max_by_key(|(depth, _)| *depth)
            .unwrap_or((0, post)))
    })
}
