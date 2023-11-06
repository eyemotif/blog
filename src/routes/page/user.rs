use crate::blog::UserID;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;

pub async fn get(Path(user_id): Path<UserID>) -> Result<Html<String>, StatusCode> {
    todo!("page/user {user_id}")
}
