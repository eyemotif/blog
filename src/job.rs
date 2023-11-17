use crate::blog::Post;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PostJob {
    AddText,
    Thumbnails,
}

pub fn create_thumbs(post: &Post) {
    let post_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post.id);

    for image_name in &post.images {
        let image = match image::io::Reader::open(post_path.join(&image_name)) {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error reading image {image_name} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };
        let image = match image.with_guessed_format() {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error guessing format for image {image_name} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };
        let image = match image.decode() {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error decoding image {image_name} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };

        // actual processing of the image
        let thumb_max_size = 256;
        let image =
            image.resize_to_fill(1024, thumb_max_size, image::imageops::FilterType::Lanczos3);
        let image = image.crop_imm(
            (image.width() - thumb_max_size) / 2,
            (image.height() - thumb_max_size) / 2,
            thumb_max_size,
            thumb_max_size,
        );

        match image.save(post_path.join(format!("{image_name}.thumb"))) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Error writing image {image_name} for post {}: {err}",
                    post.id
                );
            }
        }
    }
}
