use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use crate::util;
use crate::xattr::{list_xattrs, remove_xattr, set_xattr};
use crate::{Error, WUTAG_NAMESPACE};

fn wutag_timestamp() -> String {
    format!(
        "{}.{}",
        WUTAG_NAMESPACE,
        chrono::offset::Utc::now().timestamp()
    )
}

/// Tags the file at the given `path` with `tag`. If the tag exists returns an error.
pub fn tag_file<P, S>(path: P, tag: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    for _tag in list_tags(path.as_ref())? {
        if _tag == tag.as_ref() {
            return Err(Error::TagExists);
        }
    }
    set_xattr(path, wutag_timestamp().as_str(), tag.as_ref())
}

/// Lists tags of the file at the given `path`.
pub fn list_tags<P>(path: P) -> Result<Vec<String>, Error>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        attrs
            .into_iter()
            .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
            .map(|(_, val)| val)
            .collect::<Vec<String>>()
    })
}

/// Lists tags ofthe file at the given `path` as a [BTreeSet](BTreeSet).
pub fn list_tags_btree<P>(path: P) -> Result<BTreeSet<String>, Error>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        attrs
            .into_iter()
            .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
            .map(|(_, val)| val)
            .collect::<BTreeSet<String>>()
    })
}

/// Removes the `tag` of the file at the given `path`. If the tag doesn't exists returns
/// [Error::TagTagNotFound](Error)
pub fn remove_tag<P, S>(path: P, tag: S) -> Result<(), Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    for (key, val) in list_xattrs(path.as_ref())? {
        // make sure to only remove attributes corresponding to this namespace
        if val == tag.as_ref() && key.starts_with(WUTAG_NAMESPACE) {
            return remove_xattr(path, key);
        }
    }

    Err(Error::TagNotFound)
}

/// Clears all tags of the file at the given `path`.
pub fn clear_tags<P>(path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    for (key, _) in list_xattrs(path.as_ref())?
        .iter()
        .filter(|(key, _)| key.starts_with(WUTAG_NAMESPACE))
    {
        remove_xattr(path.as_ref(), key)?;
    }

    Ok(())
}

/// Searches for files with the specified tags in the current directory. If `recursive` is set to
/// `true` recursively follows all subdirectories. If `path` is provided the search will start at
/// the location that it points to.
///
/// Returns a list of paths of files that contain the provided set of tags.
pub fn search_files_with_tags<Ts, P>(
    tags: Ts,
    recursive: bool,
    path: Option<P>,
) -> Result<Vec<PathBuf>, Error>
where
    Ts: IntoIterator<Item = String>,
    P: AsRef<Path>,
{
    let tags = tags.into_iter().collect::<BTreeSet<_>>();
    let mut files = Vec::new();

    let dir = if let Some(path) = path {
        path.as_ref().to_string_lossy().to_string()
    } else {
        env::current_dir()?.as_path().to_string_lossy().to_string()
    };

    for entry in util::glob_walker(dir.as_str(), "**/*", recursive)? {
        if let Ok(entry) = entry {
            if let Ok(_tags) = list_tags_btree(entry.path()) {
                if !tags.is_subset(&_tags) {
                    // File doesn't have all tags
                    continue;
                }

                files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(files)
}
