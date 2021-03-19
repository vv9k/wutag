use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::util;
use wutag_core::{
    tags::{DirEntryExt, Tag},
    Result,
};

/// Searches for files with the specified tags in the location specified by `path`. If `max_depth`
/// is provided overrides [DEFAULT_MAX_DEPTH](crate::DEFAULT_MAX_DEPTH) to `max_depth` value. If
/// `any` is set to `true` all entries containing at least one of the provided tags will be
/// returned.
///
/// Returns a list of paths of files that contain the provided set of tags.
pub fn search_files_with_tags<Ts, P>(
    tags: Ts,
    path: P,
    max_depth: Option<usize>,
    any: bool,
) -> Result<Vec<PathBuf>>
where
    Ts: IntoIterator<Item = String>,
    P: AsRef<Path>,
{
    let tags = tags.into_iter().map(Tag::dummy).collect::<BTreeSet<_>>();
    let mut files = Vec::new();

    let dir = path.as_ref().to_string_lossy().to_string();

    for entry in util::glob_walker(dir.as_str(), "**/*", max_depth)? {
        if let Ok(entry) = entry {
            if let Ok(_tags) = entry.list_tags_btree() {
                if any {
                    for tag in &tags {
                        if _tags.contains(tag) {
                            files.push(entry.path().to_path_buf());
                            break;
                        }
                    }
                } else {
                    if !tags.is_subset(&_tags) {
                        // File doesn't have all tags
                        continue;
                    }

                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    Ok(files)
}
