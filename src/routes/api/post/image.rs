use crate::blog::PostID;
use axum::body::StreamBody;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio_util::io::ReaderStream;

pub(super) async fn get(
    Path((post, image)): Path<(PostID, String)>,
) -> Result<Response, StatusCode> {
    let image_file_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post)
        .join(&image);

    let file = match tokio::fs::File::open(&image_file_path).await {
        Ok(it) => it,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(StatusCode::NOT_FOUND);
            }
            eprintln!("Error reading image {image} for post {post}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let stream = ReaderStream::new(file);
    let stream = StreamBody::new(stream);

    if let Some(mime_guess) = new_mime_guess::from_path(&image_file_path).first() {
        Ok(([("Content-Type", mime_guess.to_string())], stream).into_response())
    } else {
        Ok(stream.into_response())
    }
}
