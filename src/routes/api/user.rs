use crate::blog::{User, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

pub(super) async fn get(Path(username): Path<String>) -> Result<Json<User>, StatusCode> {
    let file = match tokio::fs::read(
        std::path::Path::new(STORE_PATH)
            .join("user")
            .join(format!("{username}.json")),
    )
    .await
    {
        Ok(it) => it,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(StatusCode::NOT_FOUND);
            } else {
                eprintln!("Error reading user {username}: {err}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    let user = match serde_json::from_slice(&file) {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Error deserializing user {username}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(user))
}
