use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::utils;

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
    fs::create_dir_all(&dir)?;
    let filename = config_file_name()?;
    match utils::open_file(&filename)? {
        None => {
            let def = Config::default();
            write_config(&def)?;
            Ok(def)
        }
        Some(file) => serde_yaml::from_reader(file).map_err(|_| Error::MalformedFile(filename)),
    }
}

fn write_config(config: &Config) -> Result<()> {
    let filename = config_file_name()?;
    let file = utils::create_file(&filename)?;
    Ok(serde_yaml::to_writer(file, config)?)
}

fn config_file_name() -> Result<PathBuf> {
    Ok(project_dir()?.config_dir().join("config.yml"))
}

/// Get project directory
pub fn project_dir() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "Sam Tay", "so").ok_or_else(|| Error::ProjectDir)
}

pub fn set_api_key(key: String) -> Result<()> {
    let mut cfg = user_config()?;
    cfg.api_key = Some(key);
    write_config(&cfg)
}
