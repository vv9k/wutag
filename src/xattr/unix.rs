#![cfg(unix)]
#[cfg(target_os = "macos")]
use libc::XATTR_NOFOLLOW;
use libc::{getxattr, listxattr, removexattr, setxattr, XATTR_CREATE};
#[cfg(target_os = "linux")]
use libc::{lgetxattr, llistxattr, lremovexattr, lsetxattr};
use std::ffi::{CStr, CString, OsStr};
use std::io;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use crate::Error;

/// Sets the value of the extended attribute identified by `name` and associated with the given `path` in the
/// filesystem.
pub fn set_xattr<P, S>(path: P, name: S, value: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let size = value.as_ref().as_bytes().len();

    _set_xattr(path.as_ref(), name.as_ref(), value.as_ref(), size, false)
}

/// Retrieves the value of the extended attribute identified by `name` and associated with the given
/// `path` in the filesystem.
pub fn get_xattr<P, S>(path: P, name: S) -> Result<String, Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _get_xattr(path.as_ref(), name.as_ref(), false)
}

/// Retrieves a list of all extended attributes with their values associated with the given `path`
/// in the filesystem.
pub fn list_xattrs<P>(path: P) -> Result<Vec<(String, String)>, Error>
where
    P: AsRef<Path>,
{
    _list_xattrs(path.as_ref(), false)
}

/// Removes the extended attribute identified by `name` and associated with the given `path` in the
/// filesystem.
pub fn remove_xattr<P, S>(path: P, name: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _remove_xattr(path.as_ref(), name.as_ref(), false)
}

/// Provides identical functionality to [`set_xattr`](set_xattr) except in the case of a symbolic
/// link where the extended attribute is set on the link itself, not the file that it refers to.
pub fn set_link_xattr<P, S>(path: P, name: S, value: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let size = value.as_ref().as_bytes().len();

    _set_xattr(path.as_ref(), name.as_ref(), value.as_ref(), size, true)
}

/// Provides identical functionality to [`get_xattr`](get_xattr) except in the case of a symbolic
/// link where the extended attribute is retrieved from the link not the file that it refers to.
pub fn get_link_xattr<P, S>(path: P, name: S) -> Result<String, Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _get_xattr(path.as_ref(), name.as_ref(), true)
}

/// Provides identical functionality to [`list_xattrs`](list_xattrs) except in the case of a symbolic
/// link where the list of extended attributes is retrieved from the link no the file it refers to.
pub fn list_link_xattrs<P>(path: P) -> Result<Vec<(String, String)>, Error>
where
    P: AsRef<Path>,
{
    _list_xattrs(path.as_ref(), true)
}

/// Provides identical functionality to [`remove_xattr`](remove_xattr) except in the case of a symbolic
/// link where the value of the extended attribute is removed from the link not the file that it
/// refers to.
pub fn remove_link_xattr<P, S>(path: P, name: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    _remove_xattr(path.as_ref(), name.as_ref(), true)
}

//################################################################################
// Wrappers
//################################################################################

#[cfg(target_os = "linux")]
unsafe fn __getxattr(
    path: *const i8,
    name: *const i8,
    value: *mut c_void,
    size: usize,
    symlink: bool,
) -> isize {
    let func = if symlink { lgetxattr } else { getxattr };

    func(path, name, value, size)
}

#[cfg(target_os = "macos")]
unsafe fn __getxattr(
    path: *const i8,
    name: *const i8,
    value: *mut c_void,
    size: usize,
    symlink: bool,
) -> isize {
    let opts = if symlink { XATTR_NOFOLLOW } else { 0 };

    getxattr(path, name, value, size, 0, opts)
}

#[cfg(target_os = "linux")]
unsafe fn __setxattr(
    path: *const i8,
    name: *const i8,
    value: *const c_void,
    size: usize,
    symlink: bool,
) -> isize {
    let func = if symlink { lsetxattr } else { setxattr };

    func(path, name, value, size, XATTR_CREATE) as isize
}

#[cfg(target_os = "macos")]
unsafe fn __setxattr(
    path: *const i8,
    name: *const i8,
    value: *const c_void,
    size: usize,
    symlink: bool,
) -> isize {
    let opts = if symlink { XATTR_NOFOLLOW } else { 0 };

    setxattr(path, name, value, size, 0, opts | XATTR_CREATE) as isize
}

#[cfg(target_os = "linux")]
unsafe fn __removexattr(path: *const i8, name: *const i8, symlink: bool) -> isize {
    let func = if symlink { lremovexattr } else { removexattr };

    func(path, name) as isize
}

#[cfg(target_os = "macos")]
unsafe fn __removexattr(path: *const i8, name: *const i8, symlink: bool) -> isize {
    let opts = if symlink { XATTR_NOFOLLOW } else { 0 };

    removexattr(path, name, opts) as isize
}

#[cfg(target_os = "linux")]
unsafe fn __listxattr(path: *const i8, list: *mut i8, size: usize, symlink: bool) -> isize {
    let func = if symlink { llistxattr } else { listxattr };

    func(path, list, size) as isize
}

#[cfg(target_os = "macos")]
unsafe fn __listxattr(path: *const i8, list: *mut i8, size: usize, symlink: bool) -> isize {
    let opts = if symlink { XATTR_NOFOLLOW } else { 0 };

    listxattr(path, list, size, opts | XATTR_CREATE) as isize
}

//################################################################################
// Impl
//################################################################################

fn _remove_xattr(path: &Path, name: &str, symlink: bool) -> Result<(), Error> {
    let path = CString::new(path.to_string_lossy().as_bytes())?;
    let name = CString::new(name.as_bytes())?;

    unsafe {
        let ret = __removexattr(path.as_ptr(), name.as_ptr(), symlink);
        if ret != 0 {
            return Err(Error::from(io::Error::last_os_error()));
        }
    }

    Ok(())
}

fn _set_xattr(
    path: &Path,
    name: &str,
    value: &str,
    size: usize,
    symlink: bool, // if provided path is a symlink set the attribute on the symlink not the file/directory it points to
) -> Result<(), Error> {
    let path = CString::new(path.to_string_lossy().as_bytes())?;
    let name = CString::new(name.as_bytes())?;
    let value = CString::new(value.as_bytes())?;

    unsafe {
        let ret = __setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr() as *const c_void,
            size,
            symlink,
        );

        if ret != 0 {
            return Err(Error::from(io::Error::last_os_error()));
        }
    }

    Ok(())
}

fn _get_xattr(path: &Path, name: &str, symlink: bool) -> Result<String, Error> {
    let path = CString::new(path.to_string_lossy().as_bytes())?;
    let name = CString::new(name.as_bytes())?;
    let size = get_xattr_size(path.as_c_str(), name.as_c_str(), symlink)?;
    let mut buf = Vec::<u8>::with_capacity(size);
    let buf_ptr = buf.as_mut_ptr();

    mem::forget(buf);

    let ret = unsafe {
        __getxattr(
            path.as_ptr(),
            name.as_ptr(),
            buf_ptr as *mut c_void,
            size,
            symlink,
        )
    };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    let ret = ret as usize;

    if ret != size {
        return Err(Error::AttrsChanged);
    }

    let buf = unsafe { Vec::from_raw_parts(buf_ptr, ret, size) };

    Ok(unsafe { CString::from_vec_unchecked(buf) }
        .to_string_lossy()
        .to_string())
}

fn _list_xattrs(path: &Path, symlink: bool) -> Result<Vec<(String, String)>, Error> {
    let cpath = CString::new(path.to_string_lossy().as_bytes())?;
    let raw = list_xattrs_raw(cpath.as_c_str(), symlink)?;
    let keys = parse_xattrs(&raw)?;

    let mut attrs = Vec::new();

    for key in keys {
        attrs.push((key.clone(), _get_xattr(path, key.as_str(), symlink)?));
    }

    Ok(attrs)
}

//################################################################################
// Other
//################################################################################

fn get_xattr_size(path: &CStr, name: &CStr, symlink: bool) -> Result<usize, Error> {
    let ret = unsafe { __getxattr(path.as_ptr(), name.as_ptr(), ptr::null_mut(), 0, symlink) };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    Ok(ret as usize)
}

fn get_xattrs_list_size(path: &CStr, symlink: bool) -> Result<usize, Error> {
    let path = path.as_ref();

    let ret = unsafe { __listxattr(path.as_ptr(), ptr::null_mut(), 0, symlink) };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    Ok(ret as usize)
}

fn list_xattrs_raw(path: &CStr, symlink: bool) -> Result<Vec<u8>, Error> {
    let size = get_xattrs_list_size(path, symlink)?;
    let mut buf = Vec::<u8>::with_capacity(size);
    let buf_ptr = buf.as_mut_ptr();

    mem::forget(buf);

    let ret = unsafe { __listxattr(path.as_ptr(), buf_ptr as *mut c_char, size, symlink) };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    let ret = ret as usize;

    if ret != size {
        return Err(Error::AttrsChanged);
    }

    // its safe to construct a Vec here because original pointer to buf is forgotten
    // and the size of return buffer is verified against original size
    unsafe { Ok(Vec::from_raw_parts(buf_ptr, ret, size)) }
}

fn parse_xattrs(input: &[u8]) -> Result<Vec<String>, Error> {
    let mut it = input.iter().enumerate();
    let mut keys = Vec::new();
    let mut start = 0;

    while let Some((i, ch)) = it.next() {
        if *ch == b'\0' {
            keys.push(
                OsStr::from_bytes(&input[start..i])
                    .to_string_lossy()
                    .to_string(),
            );
            start += i - start + 1;
        }
    }

    Ok(keys)
}

#[test]
fn parses_xattrs_from_raw() {
    let raw = &[
        117, 115, 101, 114, 46, 107, 101, 121, 49, 0, 117, 115, 101, 114, 46, 107, 101, 121, 50, 0,
        117, 115, 101, 114, 46, 107, 101, 121, 51, 0, 115, 101, 99, 117, 114, 105, 116, 121, 46,
        116, 101, 115, 116, 105, 110, 103, 0,
    ];

    let attrs = parse_xattrs(raw).unwrap();
    let mut it = attrs.iter();

    assert_eq!(it.next(), Some(&"user.key1".to_string()));
    assert_eq!(it.next(), Some(&"user.key2".to_string()));
    assert_eq!(it.next(), Some(&"user.key3".to_string()));
    assert_eq!(it.next(), Some(&"security.testing".to_string()));
}
