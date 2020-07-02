use crate::error::{Error, PermissionType, Result};
use std::fs::File;
use std::io::ErrorKind;
use std::path::PathBuf;

pub fn open_file(filename: &PathBuf) -> Result<Option<File>> {
    File::open(filename).map(Some).or_else(|e| match e {
        e if e.kind() == ErrorKind::NotFound => Ok(None),
        e if e.kind() == ErrorKind::PermissionDenied => {
            Err(Error::Permissions(PermissionType::Read, filename.clone()))
        }
        e => Err(Error::from(e)),
    })
}

pub fn create_file(filename: &PathBuf) -> Result<File> {
    File::create(filename).map_err(|e| {
        if e.kind() == ErrorKind::PermissionDenied {
            Error::Permissions(PermissionType::Write, filename.clone())
        } else {
            Error::from(e)
        }
    })
}
