use serde::{Deserialize, Serialize};
pub type PostID = String;
pub type UserID = String;

pub const STORE_PATH: &str = "/Users/iris/Documents/Rust/Frith/blog/test-store";

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
    pub reply_to: Option<PostID>,
    pub replies: Vec<PostID>,
    pub quotes: Vec<PostID>,
}
