const SMALL_THUMB_SIZE: u32 = 128;
const LARGE_THUMB_SIZE: u32 = 512;

pub fn run(post: &crate::blog::Post) {
    let image_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post.id)
        .join("image");

    for image_name in &post.images {
        let raw_path = image_path.join("raw").join(image_name);
        let small_path = image_path.join("small").join(image_name);
        let large_path = image_path.join("large").join(image_name);

        let small_thumb = match create_thumb(&raw_path, SMALL_THUMB_SIZE) {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error creating small thumbnail for image {image_name} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };
        let large_thumb = match create_thumb(&raw_path, LARGE_THUMB_SIZE) {
            Ok(it) => it,
            Err(err) => {
                eprintln!(
                    "Error creating large thumbnail for image {image_name} for post {}: {err}",
                    post.id
                );
                continue;
            }
        };

        if let Some(small_thumb) = small_thumb {
            match small_thumb.save(&small_path) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!(
                        "Error writing small thumbnail for image {image_name} for post {}: {err}",
                        post.id
                    );
                }
            }
        } else {
            match std::fs::copy(&raw_path, &small_path) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!(
                        "Error writing small copy for image {image_name} for post {}: {err}",
                        post.id
                    );
                }
            }
        }
        if let Some(large_thumb) = large_thumb {
            match large_thumb.save(&large_path) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!(
                        "Error writing large thumbnail for image {image_name} for post {}: {err}",
                        post.id
                    );
                }
            }
        } else {
            match std::fs::copy(&raw_path, &large_path) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!(
                        "Error writing large copy for image {image_name} for post {}: {err}",
                        post.id
                    );
                }
            }
        }
    }
}

// TODO: better result type
fn create_thumb(
    image_path: &std::path::Path,
    max_size: u32,
) -> Result<Option<image::DynamicImage>, Box<dyn std::error::Error>> {
    let image = image::io::Reader::open(image_path)?;
    let image = image.with_guessed_format()?;

    // TODO: animated formats
    match image.format().expect("image should have format") {
        image::ImageFormat::Png => (),
        image::ImageFormat::Gif => return Ok(None),
        image::ImageFormat::WebP => (),
        _ => (),
    }

    let image = image.decode()?;
    Ok(Some(create_thumb_static(image, max_size)))
}

fn create_thumb_static(image: image::DynamicImage, max_size: u32) -> image::DynamicImage {
    let (width, height) = (image.width(), image.height());

    if height <= max_size && width <= max_size {
        image
    } else {
        image.resize_to_fill(max_size, max_size, image::imageops::Lanczos3)
    }
}
