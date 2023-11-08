use crate::blog::{PostID, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

pub(super) async fn get(
    Path(post_id): Path<PostID>,
) -> Result<Json<crate::blog::Post>, StatusCode> {
    let file = match tokio::fs::read(
        std::path::Path::new(STORE_PATH)
            .join(&post_id)
            .join("meta.json"),
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

    let post = match serde_json::from_slice(&file) {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Error deserializing post {post_id} meta: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(post))
}
