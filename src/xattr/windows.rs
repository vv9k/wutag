#![cfg(windows)]
#![allow(unused_variables)]
use std::path::Path;

use crate::Result;

pub fn set_xattr<P, S>(path: P, name: S, value: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    Ok(())
}

pub fn get_xattr<P, S>(path: P, name: S) -> Result<String>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    Ok(String::new())
}

pub fn list_xattrs<P>(path: P) -> Result<Vec<(String, String)>>
where
    P: AsRef<Path>,
{
    Ok(Vec::new())
}

pub fn remove_xattr<P, S>(path: P, name: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    Ok(())
}

//################################################################################
// Impl
//################################################################################
// Blocked by `windows-rs` `bindings::windows::win32::system_services::WIN32_STREAM_ID` type having unsupported field
// type.
