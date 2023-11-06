use crate::blog::{User, UserID};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;

pub async fn get(Path(post_id): Path<UserID>) -> Result<Json<User>, StatusCode> {
    todo!("api/user");
}
