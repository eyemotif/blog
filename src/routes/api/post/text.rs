use crate::blog::{PostID, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<String, StatusCode> {
    let file = match tokio::fs::read_to_string(
        std::path::Path::new(STORE_PATH)
            .join(&post_id)
            .join("post.md"),
    )
    .await
    {
        Ok(it) => it,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(StatusCode::NOT_FOUND);
            } else {
                eprintln!("Error reading post {post_id} meta: {err}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    Ok(file)
}
