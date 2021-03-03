pub mod opt;
pub mod tags;
pub mod util;
pub mod xattr;

use std::{ffi, io, string};
use thiserror::Error;

/// Prefixed used to identify extra attributes added by wutag on files
pub const WUTAG_NAMESPACE: &str = "user.wutag";
/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 512;

#[derive(Debug, Error)]
/// Default error used throughout this crate
pub enum Error {
    #[error("tag already exists")]
    TagExists,
    #[error("tag doesn't exist")]
    TagNotFound,
    #[error("provided file doesn't exists")]
    FileNotFound,
    #[error("error: {0}")]
    Other(String),
    #[error("provided string was invalid - {0}")]
    InvalidString(#[from] ffi::NulError),
    #[error("provided path was invalid - {0}")]
    InvalidPath(#[from] globwalk::GlobError),
    #[error("provided string was not valid UTF-8")]
    Utf8ConversionFailed(#[from] string::FromUtf8Error),
    #[error("xattrs changed while getting their size")]
    AttrsChanged,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            io::ErrorKind::AlreadyExists => Error::TagExists,
            _ => match err.raw_os_error() {
                Some(61) => Error::TagNotFound,
                _ => Error::Other(err.to_string()),
            },
        }
    }
}
