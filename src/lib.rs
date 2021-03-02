pub mod opt;
mod xattr;

use std::io;
use std::path::Path;
use thiserror::Error;

pub use xattr::*;

const RUTAG_NAMESPACE: &str = "user.rutag";

#[derive(Debug, Error)]
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
    InvalidString(#[from] std::ffi::NulError),
    #[error("provided string was not valid UTF-8")]
    Utf8ConversionFailed(#[from] std::string::FromUtf8Error),
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

fn rutag_timestamp() -> String {
    format!("{}.{}", RUTAG_NAMESPACE, chrono::offset::Utc::now())
}

pub fn tag_file<P, S>(path: P, tag: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    set_xattr(path, rutag_timestamp().as_str(), tag.as_ref())
}

pub fn list_tags<P>(path: P) -> Result<Vec<String>, Error>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        attrs
            .into_iter()
            .filter(|(key, _)| key.starts_with(RUTAG_NAMESPACE))
            .map(|(_, val)| val)
            .collect::<Vec<String>>()
    })
}

pub fn remove_tag<P, S>(path: P, tag: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    for (key, val) in list_xattrs(path.as_ref())? {
        // make sure to only remove attributes corresponding to this namespace
        if val == tag.as_ref() && key.starts_with(RUTAG_NAMESPACE) {
            return remove_xattr(path, key);
        }
    }

    Err(Error::TagNotFound)
}

pub fn clear_tags<P>(path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    for (key, _) in list_xattrs(path.as_ref())?
        .iter()
        .filter(|(key, _)| key.starts_with(RUTAG_NAMESPACE))
    {
        remove_xattr(path.as_ref(), key)?;
    }

    Ok(())
}
