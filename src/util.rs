use colored::{ColoredString, Colorize};
use globwalk::{DirEntry, GlobWalker, GlobWalkerBuilder};
use std::fmt::Display;
use std::path::Path;

use crate::DEFAULT_MAX_DEPTH;
use wutag_core::{tags::Tag, Error, Result};

pub fn fmt_err<E: Display>(err: E) -> String {
    format!("{} {}", "ERROR".red().bold(), format!("{}", err).white())
}

pub fn fmt_ok<S: AsRef<str>>(msg: S) -> String {
    format!("{} {}", "OK".green().bold(), msg.as_ref().white())
}

pub fn fmt_path<P: AsRef<Path>>(path: P) -> String {
    format!("{}", path.as_ref().display().to_string().bold().blue())
}

pub fn fmt_tag(tag: &Tag) -> ColoredString {
    tag.name().color(*tag.color()).bold()
}

/// Returns a GlobWalker instance with base path set to `base_path` and pattern to `pattern`. If
/// max_depth is specified the GlobWalker will have it's max depth set to its value, otherwise max
/// depth will be [DEFAULT_MAX_DEPTH](DEFAULT_MAX_DEPTH).
pub fn glob_walker<S>(dir: S, pattern: S, max_depth: Option<usize>) -> Result<GlobWalker>
where
    S: AsRef<str>,
{
    let mut builder = GlobWalkerBuilder::new(dir.as_ref(), pattern.as_ref());

    if let Some(max_depth) = max_depth {
        builder = builder.max_depth(max_depth);
    } else {
        builder = builder.max_depth(DEFAULT_MAX_DEPTH);
    }
    builder.build().map_err(Error::from)
}

/// Utility function that executes the function `f` on all directory entries that are Ok, by
/// default ignores all errors.
pub fn glob_ok<P, F>(pattern: &str, base_path: P, max_depth: Option<usize>, mut f: F) -> Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&DirEntry),
{
    let base_path = base_path.as_ref().to_string_lossy().to_string();

    for entry in glob_walker(base_path.as_str(), pattern, max_depth)?.flatten() {
        f(&entry);
    }

    Ok(())
}
