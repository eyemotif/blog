use crate::blog::{PostID, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<Response, StatusCode> {
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

    let html = match tokio::task::spawn_blocking(move || {
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(&arena, &file, &comrak::Options::default());

        let mut html = Vec::new();
        comrak::format_html(root, &comrak::Options::default(), &mut html)?;
        std::io::Result::Ok(html)
    })
    .await
    .expect("task should not panic")
    {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Couldn't post Markdown for post {post_id}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(([("Content-Type", "text/markdown")], Html(html)).into_response())
}
