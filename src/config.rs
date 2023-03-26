use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::error::{Error, Result};
use crate::utils;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SearchEngine {
    DuckDuckGo,
    #[default]
    Google,
    StackExchange,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub api_key: Option<String>,
    pub limit: u16,
    pub lucky: bool,
    pub sites: Vec<String>,
    pub search_engine: SearchEngine,
    pub copy_cmd: Option<String>,
}

impl fmt::Display for SearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            SearchEngine::DuckDuckGo => "duckduckgo",
            SearchEngine::Google => "google",
            SearchEngine::StackExchange => "stackexchange",
        };
        write!(f, "{s}")
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
            copy_cmd: Some(String::from(if cfg!(target_os = "macos") {
                "pbcopy"
            } else if cfg!(target_os = "windows") {
                "clip"
            } else if cfg!(target_os = "linux") {
                "xclip -sel clip"
            } else {
                // this default makes no sense but w/e
                "wl-copy"
            })),
        }
    }
}

impl Config {
    /// Get user config (writes default if none found)
    pub fn new() -> Result<Self> {
        let project = Self::project_dir()?;
        let dir = project.config_dir();
        fs::create_dir_all(dir)?;
        let filename = Self::config_file_path()?;

        match utils::open_file(&filename)? {
            None => {
                let def = Config::default();
                def.write()?;
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

    // TODO This looks odd when refactoring to associate functions under Config; perhaps this
    // shouldn't be a CLI opt? Maybe a generic --save-config based on current opts?
    pub fn set_api_key(key: String) -> Result<()> {
        let mut cfg = Self::new()?;
        cfg.api_key = Some(key);
        cfg.write()
    }

    /// Get project directory
    pub fn project_dir() -> Result<ProjectDirs> {
        ProjectDirs::from("io", "Sam Tay", "so").ok_or(Error::ProjectDir)
    }

    // TODO consider switching to .toml to be consistent with colors.toml
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::project_dir()?.config_dir().join("config.yml"))
    }

    /// Get theme file path; if it doesn't exist yet, create it with defaults.
    pub fn theme_file_path() -> Result<PathBuf> {
        let name = Self::project_dir()?.config_dir().join("colors.toml");
        if !name.as_path().exists() {
            let mut file = utils::create_file(&name)?;
            file.write_all(include_bytes!("../themes/default.toml"))?;
        }
        Ok(name)
    }

    fn write(&self) -> Result<()> {
        let filename = Self::config_file_path()?;
        let file = utils::create_file(&filename)?;
        Ok(serde_yaml::to_writer(file, &self)?)
    }

    pub fn get_copy_cmd(&self) -> Option<Command> {
        let copy_cmd_str = self.copy_cmd.as_ref()?;
        let mut pieces = copy_cmd_str.split_whitespace();
        let mut cmd = Command::new(pieces.next()?);
        cmd.args(pieces).stdin(Stdio::piped());
        Some(cmd)
    }
}
