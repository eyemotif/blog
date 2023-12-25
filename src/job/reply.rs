use crate::blog::Post;

pub async fn run(post: &Post) {
    let Some(parent_id) = post.reply_to.as_ref() else {
        return;
    };

    let parent_path = std::path::Path::new(crate::blog::STORE_PATH)
        .join("post")
        .join(parent_id)
        .join("meta.json");

    let parent_meta = match tokio::fs::read(&parent_path).await {
        Ok(it) => it,
        Err(err) => {
            eprintln!(
                "Error reading file for parent post {parent_id} of post {}: {err}",
                post.id
            );
            return;
        }
    };

    let mut parent_meta =
        serde_json::from_slice::<Post>(&parent_meta).expect("stored post should deserialize");
    parent_meta.replies.push(post.id.clone());

    let parent_meta = serde_json::to_vec(&parent_meta).expect("post should serialize");
    match tokio::fs::write(&parent_path, &parent_meta).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!(
                "Error writing file for parent post {parent_id} of post {}: {err}",
                post.id
            );
        }
    }
}
