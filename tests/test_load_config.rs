#[cfg(test)]
mod tests {
    use web_rwkv_axum::config::ModelConfig;

    fn get_config() -> ModelConfig {
        let config = include_str!("./test_parse_config.toml");
        toml::from_str(config).unwrap()
    }

    #[tokio::test]
    async fn test_tokenizer() {
        let config = get_config();
        let tokenizer = config.tokenizer.load_tokenizer().await.unwrap();
        let tokens = tokenizer.encode("sussy_baka".as_bytes()).unwrap();
        assert_eq!(
            "sussy_baka",
            String::from_utf8(tokenizer.decode(&tokens).unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn test_model() {
        let config = get_config();
        let context = config.model.create_context().await.unwrap();
        let _ = config.model.load_model(&context).await.unwrap();
    }
}
