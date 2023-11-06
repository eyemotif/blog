use axum::response::Html;

pub async fn get() -> Html<String> {
    Html(String::from("Hello, world!"))
}
