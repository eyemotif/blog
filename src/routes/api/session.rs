use crate::blog::SessionID;
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

pub(super) async fn post(
    State(state): SharedState,
    Json(login_credentials): Json<LoginCredentials>,
) -> Result<SessionID, StatusCode> {
    let auth =
        match crate::auth::Auth::validate(&login_credentials.username, login_credentials.password)
            .await
        {
            Ok(Some(it)) => it,
            Ok(None) => return Err(StatusCode::UNAUTHORIZED),
            Err(err) => {
                eprintln!(
                    "Error validating credentials for user {:?}: {err}",
                    login_credentials.username
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    Ok(state.create_session(login_credentials.username, auth).await)
}
