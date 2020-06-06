use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::fs::File;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub limit: u16,
    pub site: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: None,
            limit: 20,
            site: String::from("stackoverflow"),
        }
    }
}

/// Get user config (writes default if none found)
pub fn user_config() -> Config {
    let project = project_dir();
    let dir = project.config_dir();
    fs::create_dir_all(&dir).unwrap(); // TODO bubble to main
    let filename = dir.join("config.yml");
    match File::open(&filename) {
        Err(_) => {
            let file = File::create(&filename).unwrap();
            let def = Config::default();
            serde_yaml::to_writer(file, &def).unwrap();
            def
        }
        Ok(file) => serde_yaml::from_reader(file).expect(&format!(
            "Local config corrupted; try removing it `rm {}`",
            filename.display()
        )),
    }
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
