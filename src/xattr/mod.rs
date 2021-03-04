//! Safe and os-agnostic(TODO) wrappers for manipulating extra attributes
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::{get_link_xattr, list_link_xattrs, remove_link_xattr, set_link_xattr};
#[cfg(unix)]
use unix::{
    get_xattr as _get_xattr, list_xattrs as _list_xattrs, remove_xattr as _remove_xattr,
    set_xattr as _set_xattr,
};
#[cfg(windows)]
pub use windows::{get_xattr as _get_xattr, list_xattrs as _list_xattrs, remove_xattr as _remove_xattr, set_xattr as _set_xattr};

use crate::Error;
use std::path::Path;

pub fn set_xattr<P, S>(path: P, name: S, value: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _set_xattr(path, name, value)
}

pub fn get_xattr<P, S>(path: P, name: S) -> Result<String, Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _get_xattr(path, name)
}

pub fn list_xattrs<P>(path: P) -> Result<Vec<(String, String)>, Error>
where
    P: AsRef<Path>,
{
    _list_xattrs(path)
}

pub fn remove_xattr<P, S>(path: P, name: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _remove_xattr(path, name)
}
