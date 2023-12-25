use crate::state::NestedRouter;
use axum::routing::put;

mod add;
mod leave;
mod revoke;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/add", put(add::put))
        .route("/revoke", put(revoke::put))
        .route("/leave", put(leave::put))
}
