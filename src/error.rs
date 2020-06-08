use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    #[allow(dead_code)]
    pub kind: ErrorKind,
    pub error: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    Malformed,
    StackExchange,
    Permissions,
    OperatingSystem,
    Panic,
    EmptySites,
    NoResults,
    Termimad(termimad::Error),
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error {
            kind: ErrorKind::Panic,
            error: String::from(err),
        }
    }
}

impl From<termimad::Error> for Error {
    fn from(err: termimad::Error) -> Self {
        Error {
            kind: ErrorKind::Termimad(err),
            error: String::from(""),
        }
    }
}

// TODO add others
impl Error {
    pub fn malformed(path: &PathBuf) -> Self {
        Error {
            kind: ErrorKind::Malformed,
            error: format!("File `{}` is malformed; try removing it.", path.display()),
        }
    }
    pub fn se(err: String) -> Self {
        Error {
            kind: ErrorKind::StackExchange,
            error: err,
        }
    }
    pub fn create_dir(path: &PathBuf) -> Self {
        Error {
            kind: ErrorKind::Permissions,
            error: format!(
                "Couldn't create directory `{}`; please check the permissions
                on the parent directory",
                path.display()
            ),
        }
    }
    pub fn create_file(path: &PathBuf) -> Self {
        Error {
            kind: ErrorKind::Permissions,
            error: format!(
                "Couldn't create file `{}`; please check the directory permissions",
                path.display()
            ),
        }
    }
    pub fn write_file(path: &PathBuf) -> Self {
        Error {
            kind: ErrorKind::Permissions,
            error: format!(
                "Couldn't write to file `{}`; please check its permissions",
                path.display()
            ),
        }
    }
    pub fn os(err: &str) -> Self {
        Error {
            kind: ErrorKind::OperatingSystem,
            error: String::from(err),
        }
    }
    pub fn no_results() -> Self {
        Error {
            kind: ErrorKind::NoResults,
            error: String::from("Sorry, no answers found for your question. Try another query."),
        }
    }
}
