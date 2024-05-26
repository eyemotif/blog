use crate::blog::SessionID;

#[derive(Debug, Clone)]
pub struct Session {
    pub for_username: String,
    pub expires_at: std::time::Instant,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        std::time::Instant::now() < self.expires_at
    }
}

impl super::State {
    pub async fn get_session(&self, session_id: &SessionID) -> Option<Session> {
        let sessions = self.sessions.write().await;
        let session = sessions.get(session_id)?;

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
            expires_at: std::time::Instant::now() + crate::blog::SESSION_TTL,
        };

        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.is_valid());
        sessions.insert(session_id.clone(), new_session);

        session_id
    }
}
