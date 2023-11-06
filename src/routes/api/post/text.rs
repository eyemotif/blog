use crate::blog::PostID;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Response;

pub async fn get(Path(post_id): Path<PostID>) -> Result<Response, StatusCode> {
    todo!("api/post/text");
}
