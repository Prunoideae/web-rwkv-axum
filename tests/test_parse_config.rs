#[cfg(test)]
mod tests {
    use web_rwkv_axum::config::ModelConfig;

    #[test]
    fn test_config() {
        let config = include_str!("./test_parse_config.toml");
        let config: ModelConfig = toml::from_str(config).unwrap();
        println!("{:?}", config)
    }
}
