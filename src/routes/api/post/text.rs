use crate::blog::{PostID, SessionID, STORE_PATH};
use crate::state::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::Json;
use comrak::nodes::NodeValue;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct TextOptions {
    session: SessionID,
}

pub(super) async fn get(Path(post_id): Path<PostID>) -> Result<Html<Vec<u8>>, StatusCode> {
    if let Some(html) = get_text(post_id, None).await? {
        Ok(Html(html))
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

pub(super) async fn get_with_session(
    State(state): SharedState,
    Path(post_id): Path<PostID>,
    Json(request): Json<TextOptions>,
) -> Result<Html<Vec<u8>>, StatusCode> {
    let Some(session) = state.get_session(&request.session).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(html) = get_text(post_id, Some(&session.for_username)).await? {
        Ok(Html(html))
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn get_text(
    post_id: PostID,
    requesting_username: Option<&str>,
) -> Result<Option<Vec<u8>>, StatusCode> {
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

        process_nodes(root, &post.clone());

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

    let meta = super::meta::get(Path(post_id)).await?.0;

    if meta.is_private {
        if let Some(username) = requesting_username {
            let author_user = crate::routes::api::user::get(Path(meta.author_username))
                .await?
                .0;
            if author_user.members.contains(username) {
                return Ok(Some(html));
            }
        }
        Ok(None)
    } else {
        Ok(Some(html))
    }
}

fn process_nodes<'a>(node: &'a comrak::nodes::AstNode<'a>, post_id: &PostID) {
    process_node(node, post_id);
    for child in node.children() {
        process_nodes(child, post_id);
    }
}
fn process_node<'a>(node: &'a comrak::nodes::AstNode<'a>, post_id: &PostID) {
    match &mut node.data.borrow_mut().value {
        NodeValue::Image(link) => {
            process_link(link, post_id);
        }
        NodeValue::Link(link) => {
            process_link(link, post_id);
        }
        // NodeValue::BlockQuote => {
        //     println!("{:?}", node.children().collect::<Vec<_>>());
        // }
        _ => (),
    }
}

fn process_link(link: &mut comrak::nodes::NodeLink, post_id: &PostID) {
    if let Some(post_image) = link.url.strip_prefix("image:") {
        link.url = format!("/api/post/{post_id}/image/{post_image}");
    }
    if let Some(username) = link.url.strip_prefix('@') {
        link.url = format!("/user/{username}");
    }
}
