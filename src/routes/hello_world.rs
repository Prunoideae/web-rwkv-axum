use axum::response::Html;

pub async fn handler() -> Html<&'static str> {
    Html(include_str!("hello.html"))
}
