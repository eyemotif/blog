const SMALL_THUMB_SIZE: u32 = 128;
const LARGE_THUMB_SIZE: u32 = 512;

pub fn run(post: &crate::blog::Post) {
    let image_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(&post.id)
        .join("image");

    for image_name in &post.images {
        let small_thumb =
            match create_thumb(&image_path.join("small").join(image_name), SMALL_THUMB_SIZE) {
                Ok(it) => it,
                Err(err) => {
                    eprintln!(
                        "Error creating small thumbnail for image {image_name} for post {}: {err}",
                        post.id
                    );
                    continue;
                }
            };
        let large_thumb =
            match create_thumb(&image_path.join("large").join(image_name), LARGE_THUMB_SIZE) {
                Ok(it) => it,
                Err(err) => {
                    eprintln!(
                        "Error creating large thumbnail for image {image_name} for post {}: {err}",
                        post.id
                    );
                    continue;
                }
            };

        match small_thumb.save(image_path.join("small").join(image_name)) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Error writing small thumbnail for image {image_name} for post {}: {err}",
                    post.id
                );
            }
        }
        match large_thumb.save(image_path.join("large").join(image_name)) {
            Ok(()) => (),
            Err(err) => {
                eprintln!(
                    "Error writing large thumbnail for image {image_name} for post {}: {err}",
                    post.id
                );
            }
        }
    }
}

// TODO: better result type
fn create_thumb(
    image_path: &std::path::Path,
    max_size: u32,
) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    let image = image::io::Reader::open(image_path)?;
    let image = image.with_guessed_format()?;

    match image.format().expect("image should have format") {
        image::ImageFormat::Png => (), // TODO: apng
        image::ImageFormat::Gif => {
            // HACK: resize_to_fill breaks animated GIFs, so we just won't resize them
            // as theyre usually small anyway
            let image = image.decode()?;

            return Ok(image);
        }
        image::ImageFormat::WebP => (), // TODO: animated webp
        _ => (),
    }

    let image = image.decode()?;
    Ok(create_thumb_static(image, max_size))
}

fn create_thumb_static(image: image::DynamicImage, max_size: u32) -> image::DynamicImage {
    let (width, height) = (image.width(), image.height());

    if height <= max_size && width <= max_size {
        image
    } else {
        image.resize_to_fill(max_size, max_size, image::imageops::Lanczos3)
    }
}
