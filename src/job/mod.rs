pub mod thumbnails;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PostJob {
    AddText,
    Thumbnails,
}

impl PostJob {
    pub fn all_processing_jobs() -> std::collections::HashSet<Self> {
        std::collections::HashSet::from_iter([PostJob::Thumbnails])
    }
}
