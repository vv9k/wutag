use libc::{getxattr, listxattr, setxattr, XATTR_CREATE};
use std::ffi::{CStr, CString};
use std::io;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("tag already exists")]
    TagExists,
    #[error("provided file doesn't exists")]
    FileNotFound,
    #[error("unexpected error - `{0}`")]
    Other(String),
    #[error("provided string was invalid")]
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
            _ => Error::Other(err.to_string()),
        }
    }
}

pub fn set_xattr<P, S>(path: P, name: S, value: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
    let name = CString::new(name.as_ref().as_bytes())?;
    let size = value.as_ref().as_bytes().len();
    let value = CString::new(value.as_ref().as_bytes())?;

    unsafe {
        let ret = setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr() as *const c_void,
            size,
            XATTR_CREATE,
        );

        if ret != 0 {
            return Err(Error::from(io::Error::last_os_error()));
        }
    }

    Ok(())
}

pub fn get_xattr<P, S>(path: P, name: S) -> Result<String, Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
    let name = CString::new(name.as_ref().as_bytes())?;

    _get_xattr(path.as_c_str(), name.as_c_str())
}

pub fn list_xattrs<P>(path: P) -> Result<Vec<(String, String)>, Error>
where
    P: AsRef<Path>,
{
    let path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
    _list_xattrs(path.as_c_str())
}

//################################################################################
// Impl
//################################################################################

fn get_xattr_size(path: &CStr, name: &CStr) -> Result<usize, Error> {
    let path = path.as_ref();
    let name = name.as_ref();

    let ret = unsafe { getxattr(path.as_ptr(), name.as_ptr(), ptr::null_mut(), 0) };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    Ok(ret as usize)
}

fn _get_xattr(path: &CStr, name: &CStr) -> Result<String, Error> {
    let size = get_xattr_size(path, name)?;
    let mut buf = Vec::<u8>::with_capacity(size);
    let buf_ptr = buf.as_mut_ptr();

    mem::forget(buf);

    let ret = unsafe { getxattr(path.as_ptr(), name.as_ptr(), buf_ptr as *mut c_void, size) };

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

fn _list_xattrs(path: &CStr) -> Result<Vec<(String, String)>, Error> {
    let raw = list_xattrs_raw(path)?;
    let keys = parse_xattrs(&raw)?;

    let mut attrs = Vec::new();

    for key in keys {
        let _key = CString::new(key.as_str())?;
        attrs.push((key, _get_xattr(path, _key.as_c_str())?));
    }

    Ok(attrs)
}

fn get_xattrs_list_size(path: &CStr) -> Result<usize, Error> {
    let path = path.as_ref();

    let ret = unsafe { listxattr(path.as_ptr(), ptr::null_mut(), 0) };

    if ret == -1 {
        return Err(Error::from(io::Error::last_os_error()));
    }

    Ok(ret as usize)
}

fn list_xattrs_raw(path: &CStr) -> Result<Vec<u8>, Error> {
    let size = get_xattrs_list_size(path)?;
    let mut buf = Vec::<u8>::with_capacity(size);
    let buf_ptr = buf.as_mut_ptr();

    mem::forget(buf);

    let ret = unsafe { listxattr(path.as_ptr(), buf_ptr as *mut c_char, size) };

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
    let mut it = input.iter();
    let mut keys = Vec::new();
    let mut key = Vec::new();

    while let Some(ch) = it.next() {
        match ch {
            b'\0' => {
                keys.push(String::from_utf8(mem::take(&mut key))?);
            }
            _ => key.push(*ch),
        }
    }

    Ok(keys)
}
