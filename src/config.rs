use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::utils;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub limit: u16,
    pub site: String,
}

// TODO make a friender config file, like the colors.toml below
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
        file.write_all(DEFAULT_COLORS_TOML.as_bytes())?;
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

// TODO either try to respect terminal defaults, or go all in one solarized or
// tomorrow night 80s
static DEFAULT_COLORS_TOML: &str = r##"
# Every field in a theme file is optional.

shadow = false
borders = "outset" # Alternatives are "none" and "simple"

# Base colors are red, green, blue,
# cyan, magenta, yellow, white and black.
[colors]
	# There are 3 ways to select a color:
	# - The 16 base colors are selected by name:
	#       "blue", "light red", "magenta", ...
	# - Low-resolution colors use 3 characters, each <= 5:
	#       "541", "003", ...
	# - Full-resolution colors start with '#' and can be 3 or 6 hex digits:
	#       "#1A6", "#123456", ...

	# If the value is an array, the first valid
	# and supported color will be used.
	background = ["default"]

	# If the terminal doesn't support custom color (like the linux TTY),
	# non-base colors will be skipped.
	shadow     = []
	view       = "111"

	# An array with a single value has the same effect as a simple value.
	primary   = []
	secondary = "cyan"
	tertiary  = "green"

	# Hex values can use lower or uppercase.
	# (base color MUST be lowercase)
	title_primary   = ["BLUE", "yellow"] # `BLUE` will be skipped.
	title_secondary = "#ffff55"

	# Lower precision values can use only 3 digits.
	highlight          = "#F88"
	highlight_inactive = "#5555FF"
"##;
