use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

pub type PostID = String;
pub type UserID = String;
pub type SessionID = String;

pub const STORE_PATH: &str = "/Users/iris/Documents/Rust/Frith/blog/test-store";
pub const POST_ID_BYTES: usize = 16;
pub const USER_ID_BYTES: usize = 8;
pub const SESSION_ID_BYTES: usize = 32;
pub const SESSION_EXPIRED_AFTER: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserID,
    pub username: String,
    pub name: String,
    pub posts: Vec<PostID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: PostID,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reply_to: Option<PostID>,
    pub replies: Vec<PostID>,
    pub quotes: Vec<PostID>,
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
