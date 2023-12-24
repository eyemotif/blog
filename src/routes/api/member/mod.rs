use crate::state::NestedRouter;
use axum::routing::put;

mod join;
mod revoke;

pub fn route() -> NestedRouter {
    axum::Router::new()
        .route("/join", put(join::put))
        .route("/revoke", put(revoke::put))
}
