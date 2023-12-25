use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Write;

pub type PostID = String;
pub type SessionID = String;
pub type InviteID = String;

pub const STORE_PATH: &str = "/home/shared/frith-store/blog";

pub const POST_ID_BYTES: usize = 16;
pub const SESSION_ID_BYTES: usize = 32;
pub const INVITE_ID_BYTES: usize = 32;

pub const SESSION_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
pub const INCOMPLETE_POST_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60);
pub const INVITE_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 7);

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub name: String,
    // in chronological order
    pub posts: Vec<PostID>,

    pub permissions: Permissions,
    pub members: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostID,
    pub author_username: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reply_to: Option<PostID>,
    pub replies: Vec<PostID>,
    pub quotes: Vec<PostID>,
    pub in_progress: bool,
    pub images: Vec<String>,

    #[serde(default)]
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub can_create_invites: bool,
    pub can_create_posts: bool,
}

pub fn get_random_hex_string<const LEN: usize>() -> String {
    let mut bytes = [0u8; LEN];
    rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut bytes);

    bytes.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}
