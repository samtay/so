use directories;

pub struct Site {
    url: String,
    code: String,
}

pub struct Config {
    filter: String,
    sites: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        assert!(true)
    }
}
