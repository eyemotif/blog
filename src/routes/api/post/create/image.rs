use crate::blog::PostID;
use crate::routes::api::SessionQuery;
use crate::state::SharedState;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use std::os::unix::ffi::OsStrExt;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;

const IMAGE_SOCKET_TTL: Duration = Duration::from_secs(60);
const IMAGE_SOCKET_MESSAGE_TTL: Duration = Duration::from_secs(1);

pub(super) async fn post(
    State(state): SharedState,
    Path((post_id, image_name)): Path<(PostID, String)>,
    Query(query): Query<SessionQuery>,
) -> Result<String, StatusCode> {
    let Some(session) = state.get_session(&query.session).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let image_name = std::path::Path::new(&image_name);

    let mut posts_in_progress = state.posts_in_progress.write().await;
    let Some(post) = posts_in_progress.get_mut(&post_id) else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !post.meta.in_progress {
        return Err(StatusCode::NOT_FOUND);
    }
    if post.meta.author_username != session.for_username {
        return Err(StatusCode::FORBIDDEN);
    }

    let Some(image_name) = image_name.file_name() else {
        return Err(StatusCode::BAD_REQUEST);
    };
    if image_name.len() > 100 {
        // dumb filename length cap
        return Err(StatusCode::BAD_REQUEST);
    }

    let image_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post_id)
        .join(image_name);
    match tokio::fs::write(&image_path, Vec::new()).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error creating image file {image_path:?}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    post.jobs_left.insert(crate::job::PostJob::Thumbnails);

    Ok(format!(
        "/api/post/image/{post_id}/{}",
        urlencoding::encode_binary(image_name.as_bytes())
    ))
}

pub(super) async fn ws(
    Path((post_id, image_name)): Path<(PostID, String)>,
    socket: WebSocketUpgrade,
) -> Response {
    socket.on_upgrade(|socket| {
        handle_image_socket(
            socket,
            std::path::Path::new(crate::blog::STORE_PATH)
                .join("post")
                .join(post_id)
                .join(image_name),
        )
    })
}

async fn handle_image_socket(mut socket: WebSocket, image_path: std::path::PathBuf) {
    let mut image_file = match tokio::fs::OpenOptions::new()
        .append(true)
        .open(&image_path)
        .await
    {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Error opening image file {image_path:?}: {err}");
            _ = tokio::time::timeout(Duration::from_secs(5), socket.close()).await;
            return;
        }
    };
    let mut total_recv_time = Duration::ZERO;

    loop {
        let recv_start = Instant::now();
        let message_or_timeout =
            tokio::time::timeout(IMAGE_SOCKET_MESSAGE_TTL, socket.recv()).await;
        let Ok(message) = message_or_timeout else {
            close_socket(socket, String::from("Timed out")).await;
            break;
        };

        total_recv_time += recv_start.elapsed();
        if total_recv_time >= IMAGE_SOCKET_TTL {
            close_socket(socket, String::from("Timed out")).await;
            break;
        }

        let message = match message {
            Some(Ok(it)) => it,
            Some(Err(err)) => {
                eprintln!("Error handling image socket for {image_path:?}: {err}");
                close_socket(socket, String::from("Internal server error")).await;
                break;
            }
            None => break,
        };

        match message {
            axum::extract::ws::Message::Binary(data) => match image_file.write_all(&data).await {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error writing to image file {image_path:?}: {err}");
                    close_socket(socket, String::from("Internal server error")).await;
                    break;
                }
            },
            axum::extract::ws::Message::Close(_) => return,
            axum::extract::ws::Message::Ping(_)
            | axum::extract::ws::Message::Pong(_)
            | axum::extract::ws::Message::Text(_) => (),
        }
    }

    drop(image_file);
    match tokio::fs::remove_file(&image_path).await {
        Ok(()) => (),

        Err(err) => eprintln!("Could not delete image file {image_path:?}: {err}"),
    }
}

async fn close_socket(mut socket: axum::extract::ws::WebSocket, reason: String) {
    _ = tokio::time::timeout(Duration::from_secs(5), async move {
        _ = socket
            .send(axum::extract::ws::Message::Close(Some(
                axum::extract::ws::CloseFrame {
                    code: 1000,
                    reason: reason.into(),
                },
            )))
            .await;
        _ = socket.close().await;
    })
    .await;
}
