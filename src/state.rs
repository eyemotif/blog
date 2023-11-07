use crate::blog::SessionID;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = axum::extract::State<Arc<State>>;
pub type NestedRouter = axum::Router<Arc<State>>;

#[derive(Debug)]
pub struct State {
    pub sessions: RwLock<HashMap<SessionID, Session>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub for_username: String,
    pub expires_at: std::time::Instant,
}

impl State {
    pub fn new() -> State {
        State {
            sessions: RwLock::new(HashMap::new()),
        }
    }
    pub async fn get_session(&self, session_id: &SessionID) -> Option<Session> {
        let sessions = self.sessions.read().await;
        let Some(session) = sessions.get(session_id) else {
            // TODO: cleanup expired sessions
            return None;
        };

        session.is_valid().then(|| session.clone())
    }
    pub async fn create_session(
        &self,
        for_username: String,
        _auth: crate::auth::Auth,
    ) -> SessionID {
        let session_id: SessionID =
            crate::blog::get_random_hex_string::<{ crate::blog::SESSION_ID_BYTES }>();
        let new_session = Session {
            for_username,
            expires_at: std::time::Instant::now() + crate::blog::SESSION_EXPIRED_AFTER,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), new_session);

        session_id
    }
}

impl Session {
    pub fn is_valid(&self) -> bool {
        std::time::Instant::now() < self.expires_at
    }
}
