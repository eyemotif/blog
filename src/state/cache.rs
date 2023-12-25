use crate::blog::Post;

#[derive(Debug, Default)]
pub struct Cache {
    pub latest_posts: Option<Vec<Post>>,
}
