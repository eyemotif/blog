use crate::blog::PostID;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

pub async fn get(Path(post_id): Path<PostID>) -> Result<Json<crate::blog::Post>, StatusCode> {
    todo!("api/post/meta");
}
