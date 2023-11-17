use crate::blog::Post;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PostJob {
    AddText,
    Thumbnails,
}

fn create_thumb(image: image::DynamicImage, max_size: u32) -> image::DynamicImage {
    image
        .resize_to_fill(max_size * 4, max_size, image::imageops::Lanczos3)
        .crop(
            (image.width() - max_size) / 2,
            (image.height() - max_size) / 2,
            max_size,
            max_size,
        )
}

pub fn create_thumbs(post: &Post) {
    let post_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post.id);

    for image_name in &post.images {
        let image = match image::io::Reader::open(post_path.join(image_name)) {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error reading image {image_name:?} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };
        let image = match image.with_guessed_format() {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error guessing format for image {image_name:?} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };
        let image = match image.decode() {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error decoding image {image_name:?} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };

        let small_thumb = create_thumb(image.clone(), 256);
        let large_thumb = create_thumb(image, 512);

        match small_thumb.save(post_path.join(format!("{image_name:?}.thumb"))) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Error writing small thumbnail for image {image_name:?} for post {}: {err}",
                    post.id
                );
            }
        }
        match large_thumb.save(post_path.join(format!("{image_name:?}.thumb.large"))) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Error writing large thumbnail for image {image_name:?} for post {}: {err}",
                    post.id
                );
            }
        }
    }
}

impl PostJob {
    pub fn all_processing_jobs() -> std::collections::HashSet<Self> {
        std::collections::HashSet::from_iter([PostJob::Thumbnails])
    }
}
