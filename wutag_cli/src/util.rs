use colored::{ColoredString, Colorize};
use globwalk::{GlobWalker, GlobWalkerBuilder};
use std::path::{Path, PathBuf};

use crate::DEFAULT_MAX_DEPTH;
use anyhow::{Context, Result};
use wutag_core::tag::Tag;

pub fn fmt_path<P: AsRef<Path>>(path: P) -> ColoredString {
    path.as_ref().display().to_string().bold().blue()
}

pub fn fmt_tag(tag: &Tag) -> ColoredString {
    if tag.name().chars().any(|c| c.is_ascii_whitespace()) {
        format!("\"{}\"", tag.name()).color(*tag.color()).bold()
    } else {
        tag.name().color(*tag.color()).bold()
    }
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
    builder.build().context("invalid path")
}

pub fn glob_paths<P>(pattern: &str, base_path: P, max_depth: Option<usize>) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let base_path = base_path.as_ref().to_string_lossy().to_string();

    Ok(glob_walker(base_path.as_str(), pattern, max_depth)?
        .flatten()
        .map(|entry| entry.into_path())
        .collect())
}
