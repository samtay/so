use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Termimad error: {0}")]
    Termimad(#[from] termimad::Error),
    #[error("Crossterm error: {0}")]
    Crossterm(#[from] crossterm::ErrorKind),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("SerdeJson error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("SerdeYaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("Futures Join error : {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("File `{}` is malformed; try removing it", .0.display())]
    MalformedFile(PathBuf),
    #[error("Lacking {0:?} permissions on `{}`", .1.display())]
    Permissions(PermissionType, PathBuf),
    #[error("{0}")]
    StackExchange(String),
    #[error("{0}")]
    ScrapingError(String),
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
