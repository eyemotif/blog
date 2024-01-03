pub mod reply;
pub mod thumbnails;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, enum_iterator::Sequence)]
pub enum PostJob {
    /// Add the actual content of a post
    AddText,
    /// Create thumbnails for all of a post's images
    Thumbnails,
    /// Update the parent's `replies` entry
    ReplyParent,
}
