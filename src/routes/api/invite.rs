use crate::blog::{InviteID, Permissions, SessionID};
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct InviteOptions {
    session: SessionID,
    for_permissions: Permissions,
}

pub(super) async fn post(
    State(state): SharedState,
    Json(request): Json<InviteOptions>,
) -> Result<InviteID, StatusCode> {
    let Some(session) = state.get_session(&request.session).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let user = super::user::get(axum::extract::Path(session.for_username))
        .await?
        .0;
    if !user.permissions.can_create_invites {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(state.create_invite(request.for_permissions).await)
}
