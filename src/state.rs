use std::sync::Arc;

pub type SharedState = axum::extract::State<Arc<State>>;
pub type NestedRouter = axum::Router<Arc<State>>;

#[derive(Debug)]
pub struct State {}
