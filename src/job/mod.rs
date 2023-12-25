use std::ops::Not;

pub mod reply;
pub mod thumbnails;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PostJob {
    /// Add the actual content of a post
    AddText,
    /// Create thumbnails for all of a post's images
    Thumbnails,
    /// Update the parent's `replies` entry
    ReplyParent,
}

impl PostJob {
    pub fn all_possible_processing_jobs(
        post: &crate::blog::Post,
    ) -> std::collections::HashSet<Self> {
        [
            post.images.is_empty().not().then_some(PostJob::Thumbnails),
            post.reply_to.is_some().then_some(PostJob::ReplyParent),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
