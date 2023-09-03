use axum::response::Html;

pub async fn handler() -> Html<&'static str> {
    Html(
        r#"
    <h1>Hello World!</h1>
    <h2>白毛狼耳萝莉温柔地注视着你，不再言语。<h2>
    "#,
    )
}
