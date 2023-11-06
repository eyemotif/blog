use crate::blog::PostID;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;

pub async fn get(Path(post_id): Path<PostID>) -> Result<Html<String>, StatusCode> {
    todo!("page/post {post_id}")
}
