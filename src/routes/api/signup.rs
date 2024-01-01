use std::collections::HashSet;

use crate::blog::{InviteID, SessionID};
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct SignupOptions {
    invite_id: InviteID,
    username: String,
    name: String,
    password: String,
}

pub(super) async fn post(
    State(state): SharedState,
    Json(request): Json<SignupOptions>,
) -> Result<SessionID, StatusCode> {
    if !request.is_valid() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let Some(invite) = state.get_invite(&request.invite_id).await else {
        return Err(StatusCode::NOT_FOUND);
    };

    let auth = match crate::auth::Auth::write_entry(&request.username, request.password).await {
        Ok(Some(auth)) => auth,
        Ok(None) => return Err(StatusCode::CONFLICT),
        Err(err) => {
            eprintln!(
                "Error writing username and password for invite {}: {err}",
                request.invite_id
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let new_user = crate::blog::User {
        username: request.username.clone(),
        name: request.name,
        posts: Vec::new(),
        permissions: invite.for_permissions,
        members: HashSet::new(),
    };

    match tokio::fs::write(
        std::path::Path::new(crate::blog::STORE_PATH)
            .join("user")
            .join(format!("{}.json", request.username)),
        serde_json::to_vec(&new_user).expect("user should serialize"),
    )
    .await
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Error writing new user {}.json: {err}", request.username);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    state.remove_invite(&request.invite_id).await;
    let auth_session_id = state.create_session(request.username, auth).await;
    Ok(auth_session_id)
}

impl SignupOptions {
    fn is_valid(&self) -> bool {
        static USERNAME_PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

        let username_pattern = USERNAME_PATTERN.get_or_init(|| {
            regex::Regex::new(r"^[a-zA-Z0-9-_]+$").expect("constant pattern should parse")
        });

        if !username_pattern.is_match(&self.username) {
            return false;
        }

        true
    }
}
