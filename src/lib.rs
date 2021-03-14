pub mod tags;
pub mod util;
pub mod xattr;

use std::{ffi, io, string};
use thiserror::Error;

/// Prefix used to identify extra attributes added by wutag on files
pub const WUTAG_NAMESPACE: &str = "user.wutag";
/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;

#[derive(Debug, Error)]
/// Default error used throughout this crate
pub enum Error {
    #[error("tag already exists")]
    TagExists,
    #[error("tag `{0}` doesn't exist")]
    TagNotFound(String),
    #[error("tag key was invalid - {0}")]
    InvalidTagKey(String),
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
    #[error("provided color `{0}` is not a valid hex color")]
    InvalidColor(String),
    #[error("provided shell name `{0}` is not a valid shell")]
    InvalidShell(String),
    #[error("failed to serialize or deserialize tag - `{0}`")]
    TagSerDeError(#[from] serde_cbor::Error),
    #[error("failed to decode data with base64 - `{0}`")]
    Base64DecodeError(#[from] base64::DecodeError),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            io::ErrorKind::AlreadyExists => Error::TagExists,
            _ => match err.raw_os_error() {
                Some(61) => Error::TagNotFound("".to_string()),
                _ => Error::Other(err.to_string()),
            },
        }
    }
}
