use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::utils;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")] // TODO test this
pub enum SearchEngine {
    DuckDuckGo,
    Google,
    StackExchange,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub api_key: Option<String>,
    pub limit: u16,
    pub lucky: bool,
    pub sites: Vec<String>,
    pub search_engine: SearchEngine,
}

impl fmt::Display for SearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            SearchEngine::DuckDuckGo => "duckduckgo",
            SearchEngine::Google => "google",
            SearchEngine::StackExchange => "stackexchange",
        };
        write!(f, "{}", s)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        SearchEngine::DuckDuckGo
    }
}

// TODO make a friender config file, like the colors.toml below
impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: Some(String::from("8o9g7WcfwnwbB*Qp4VsGsw((")),
            limit: 20,
            lucky: true,
            sites: vec![String::from("stackoverflow")],
            search_engine: SearchEngine::default(),
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
        Some(file) => serde_yaml::from_reader(file)
            .map_err(|_| Error::MalformedFile(filename.clone()))
            .and_then(|cfg: Config| {
                if cfg.sites.is_empty() {
                    Err(Error::MalformedFile(filename))
                } else {
                    Ok(cfg)
                }
            }),
    }
}

pub fn set_api_key(key: String) -> Result<()> {
    let mut cfg = user_config()?;
    cfg.api_key = Some(key);
    write_config(&cfg)
}

/// Get project directory
pub fn project_dir() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "Sam Tay", "so").ok_or_else(|| Error::ProjectDir)
}

pub fn theme_file_name() -> Result<PathBuf> {
    let name = project_dir()?.config_dir().join("colors.toml");
    if !name.as_path().exists() {
        let mut file = utils::create_file(&name)?;
        file.write_all(include_str!("../themes/default.toml").as_bytes())?;
    }
    Ok(name)
}

fn write_config(config: &Config) -> Result<()> {
    let filename = config_file_name()?;
    let file = utils::create_file(&filename)?;
    Ok(serde_yaml::to_writer(file, config)?)
}

// TODO consider switching to .toml to be consistent with colors.toml
fn config_file_name() -> Result<PathBuf> {
    Ok(project_dir()?.config_dir().join("config.yml"))
}
