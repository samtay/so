use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;

use crate::error::{Error, Result};

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
pub fn user_config() -> Result<Config> {
    let project = project_dir()?;
    let dir = project.config_dir();
    fs::create_dir_all(&dir).map_err(|_| Error::create_dir(&dir.to_path_buf()))?;
    let filename = dir.join("config.yml");
    match File::open(&filename) {
        Err(_) => {
            let file = File::create(&filename).map_err(|_| Error::create_file(&filename))?;
            let def = Config::default();
            serde_yaml::to_writer(file, &def).map_err(|_| Error::write_file(&filename))?;
            Ok(def)
        }
        Ok(file) => serde_yaml::from_reader(file).map_err(|_| Error::malformed(&filename)),
    }
}

/// Get project directory
pub fn project_dir() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "Sam Tay", "so").ok_or_else(|| {
        Error::os(
            "Couldn't find a suitable project directory to store cache and configuration;\n\
            this application may not be supported on your operating system.",
        )
    })
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
