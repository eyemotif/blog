use crate::blog::SessionID;
use crate::state::SharedState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct MemberRevokeOptions {
    session: SessionID,
    for_username: String,
}

pub(super) async fn put(
    State(state): SharedState,
    Json(request): Json<MemberRevokeOptions>,
) -> StatusCode {
    let Some(session) = state.get_session(&request.session).await else {
        return StatusCode::UNAUTHORIZED;
    };

    let mut user = match crate::routes::api::user::get(axum::extract::Path(
        session.for_username.clone(),
    ))
    .await
    {
        Ok(Json(user)) => user,
        Err(err) => return err,
    };

    user.members.insert(request.for_username.clone());

    match tokio::fs::write(
        std::path::Path::new(crate::blog::STORE_PATH)
            .join("user")
            .join(format!("{}.json", session.for_username)),
        serde_json::to_vec(&user).expect("user should serialize"),
    )
    .await
    {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            eprintln!("Error writing user {}.json: {err}", session.for_username);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
