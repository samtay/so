pub struct Config {
    pub api_key: Option<String>,
    pub limit: u16,
    pub site: String,
}

/// Get project directory; might panic on unexpected OS
pub fn project_dir() -> ProjectDirs {
    ProjectDirs::from("io", "Sam Tay", "so").expect(
        "Couldn't find
        a suitable project directory to store cache and configuration; this
        application may not be supported on your operating system.",
    )
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
