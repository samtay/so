pub struct Config {
    pub api_key: String,
    pub limit: u16,
    pub site: String,
}

#[cfg(test)]
mod tests {
    // TODO test malformed filter string
    // TODO test malformed api key
    // for both, detect situation and print helpful error message
    #[test]
    fn test_merge_configs() {
        assert!(true)
    }
}
