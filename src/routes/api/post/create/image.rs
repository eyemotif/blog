use crate::blog::PostID;
use crate::routes::api::SessionQuery;
use crate::state::SharedState;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;

const IMAGE_SOCKET_TTL: Duration = Duration::from_secs(60);
const IMAGE_SOCKET_MESSAGE_TTL: Duration = Duration::from_secs(1);

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum ImageSocketMessage {
    GetUrl,
    End,
    Cancel,
}

pub(super) async fn post(
    State(state): SharedState,
    Path((post_id, image_name)): Path<(PostID, PathBuf)>,
    Query(query): Query<SessionQuery>,
) -> Result<String, StatusCode> {
    let Some(session) = state.get_session(&query.session).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some(image_name) = image_name.file_name() else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let post = crate::routes::api::post::meta::get(Path(post_id.clone()))
        .await?
        .0;

    if !post.in_progress {
        return Err(StatusCode::NOT_FOUND);
    }
    if post.author_username != session.for_username {
        return Err(StatusCode::FORBIDDEN);
    }

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

    Ok(urlencoding::encode_binary(&file_url_bytes(&post_id, image_name)).into_owned())
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
            break;
        };

        total_recv_time += recv_start.elapsed();
        if total_recv_time >= IMAGE_SOCKET_TTL {
            break;
        }

        let message = match message {
            Some(Ok(it)) => it,
            Some(Err(err)) => {
                eprintln!("Error handling image socket for {image_path:?}: {err}");
                break;
            }
            None => break,
        };

        match message {
            axum::extract::ws::Message::Text(text) => {
                match serde_json::from_str::<ImageSocketMessage>(&text) {
                    Ok(message) => match message {
                        // TODO: send image url
                        ImageSocketMessage::GetUrl => todo!("image url"),
                        ImageSocketMessage::End => {
                            _ = tokio::time::timeout(Duration::from_secs(5), socket.close()).await;
                            return;
                        }
                        ImageSocketMessage::Cancel => break,
                    },
                    Err(_) => break,
                }
            }
            axum::extract::ws::Message::Binary(data) => match image_file.write_all(&data).await {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error writing to image file {image_path:?}: {err}");
                    break;
                }
            },
            axum::extract::ws::Message::Close(_) => break,
            axum::extract::ws::Message::Ping(_) | axum::extract::ws::Message::Pong(_) => (),
        }
    }

    // socket never sent `end` message or sent `cancel` message
    drop(image_file);
    match tokio::fs::remove_file(&image_path).await {
        Ok(()) => (),
        Err(err) => eprintln!("Could not delete image file {image_path:?}: {err}"),
    }

    _ = tokio::time::timeout(Duration::from_secs(5), socket.close()).await;
}

fn file_url_bytes(post_id: &PostID, image_name: &std::ffi::OsStr) -> Vec<u8> {
    let mut bytes = b"/api/post/image/".to_vec();
    bytes.extend_from_slice(post_id.as_bytes());
    bytes.push(b'/');
    bytes.extend_from_slice(image_name.as_bytes());

    bytes
}
