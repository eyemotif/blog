use rand::RngCore;
use serde::{Deserialize, Serialize};

pub type PostID = String;
pub type SessionID = String;

pub const STORE_PATH: &str = "/home/shared/frith-store/blog";
pub const POST_ID_BYTES: usize = 16;
pub const SESSION_ID_BYTES: usize = 32;
pub const SESSION_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);
pub const INCOMPLETE_POST_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub name: String,
    // in chronological order
    pub posts: Vec<PostID>,
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
}

pub fn get_random_hex_string<const LEN: usize>() -> String {
    // TODO: this doesn't use a cryptographically secure randomness algorithm
    let mut bytes = [0u8; LEN];
    rand::thread_rng().fill_bytes(&mut bytes);

    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("")
}
