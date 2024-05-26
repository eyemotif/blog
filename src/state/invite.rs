use crate::blog::{InviteID, Permissions};

#[derive(Debug, Clone)]
pub struct Invite {
    pub for_permissions: Permissions,
    pub expires_at: std::time::Instant,
}

impl Invite {
    pub fn is_valid(&self) -> bool {
        std::time::Instant::now() < self.expires_at
    }
}

impl super::State {
    pub async fn get_invite(&self, invite_id: &InviteID) -> Option<Invite> {
        let invites = self.invites.read().await;
        let invite = invites.get(invite_id)?;

        invite.is_valid().then(|| invite.clone())
    }
    pub async fn remove_invite(&self, invite_id: &InviteID) -> Option<Invite> {
        self.invites.write().await.remove(invite_id)
    }

    pub async fn create_invite(&self, for_permissions: Permissions) -> InviteID {
        let invite_id: InviteID =
            crate::blog::get_random_hex_string::<{ crate::blog::INVITE_ID_BYTES }>();
        let new_invite = Invite {
            for_permissions,
            expires_at: std::time::Instant::now() + crate::blog::INVITE_TTL,
        };

        let mut invites = self.invites.write().await;
        invites.retain(|_, invite| invite.is_valid());
        invites.insert(invite_id.clone(), new_invite);

        invite_id
    }
}
