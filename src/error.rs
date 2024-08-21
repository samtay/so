use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

// TODO convert/remove this to just use anyhow
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Termimad error: {0}")]
    Termimad(#[from] termimad::Error),
    #[error("Crossterm error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("SerdeJson error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("SerdeYaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("Futures Join error : {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("File `{}` is malformed; try removing it", .0.display())]
    MalformedFile(PathBuf),
    #[error("Lacking {0:?} permissions on `{}`", .1.display())]
    Permissions(PermissionType, PathBuf),
    #[error("{0}")]
    StackExchange(String),
    #[error("{0}")]
    Scraping(String),
    #[error("Couldn't find a suitable project directory; is your OS supported?")]
    ProjectDir,
    #[error("Sorry, couldn't find any answers to your question")]
    NoResults,
}

#[derive(Debug)]
pub enum PermissionType {
    Read,
    Write,
}
