use crate::blog::{User, UserID, STORE_PATH};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;

pub async fn get(Path(user_id): Path<UserID>) -> Result<Json<User>, StatusCode> {
    let file =
        match tokio::fs::read(std::path::Path::new(STORE_PATH).join(format!("{user_id}.json")))
            .await
        {
            Ok(it) => it,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Err(StatusCode::NOT_FOUND);
                } else {
                    eprintln!("Error reading user {user_id}: {err}");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        };

    let user = match serde_json::from_slice(&file) {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Error deserializing user {user_id}: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(user))
}
