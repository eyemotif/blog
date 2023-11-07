use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;

pub async fn get(Path(username): Path<String>) -> Result<Html<String>, StatusCode> {
    todo!("page/user {username}")
}
