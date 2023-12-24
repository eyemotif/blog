use crate::state::NestedRouter;
use axum::routing::put;

mod join;
mod leave;
mod revoke;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/join", put(join::put))
        .route("/leave", put(leave::put))
        .route("/revoke", put(revoke::put))
}
