use crate::blog::Post;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PostJob {
    AddText,
    ResizeImages,
}

pub fn downsize_images(post: &Post) {
    // let image = image::io::Reader::open(path)
    todo!()
}
