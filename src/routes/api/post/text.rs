use crate::blog::{PostID, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<Response, StatusCode> {
    let file = match tokio::fs::read_to_string(
        std::path::Path::new(STORE_PATH)
            .join("post")
            .join(&post_id)
            .join("text.md"),
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

    let post = post_id.clone();
    let html = match tokio::task::spawn_blocking(move || {
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(&arena, &file, &comrak::Options::default());

        process_nodes(root, &post_id);

        let mut html = Vec::new();
        comrak::format_html(root, &comrak::Options::default(), &mut html)?;
        std::io::Result::Ok(html)
    })
    .await
    .expect("task should not panic")
    {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Couldn't post Markdown for post {post}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(([("Content-Type", "text/markdown")], Html(html)).into_response())
}

fn process_nodes<'a>(node: &'a comrak::nodes::AstNode<'a>, post_id: &PostID) {
    process_node(node, post_id);
    for child in node.children() {
        process_nodes(child, post_id);
    }
}

fn process_node<'a>(node: &'a comrak::nodes::AstNode<'a>, post_id: &PostID) {
    match &mut node.data.borrow_mut().value {
        comrak::nodes::NodeValue::Image(link) => {
            process_link(link, post_id);
        }
        comrak::nodes::NodeValue::Link(link) => {
            process_link(link, post_id);
        }
        _ => (),
    }
}

fn process_link(link: &mut comrak::nodes::NodeLink, post_id: &PostID) {
    if let Some(post_image) = link.url.strip_prefix("image:") {
        link.url = format!("/api/post/image/{post_id}/{post_image}")
    }
}
