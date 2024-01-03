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
    pub fn all_possible_processing_jobs() -> std::collections::HashSet<Self> {
        [PostJob::Thumbnails, PostJob::ReplyParent]
            .into_iter()
            .collect()
    }
}
