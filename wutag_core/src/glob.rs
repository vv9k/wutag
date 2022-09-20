use crate::{Error, Result};
use globwalk::{GlobWalker, GlobWalkerBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;

#[derive(Debug, Deserialize, Serialize)]
pub struct Glob {
    pub pattern: String,
    pub base_dir: PathBuf,
    pub max_depth: usize,
}

impl Glob {
    pub fn new(
        pattern: String,
        base_dir: Option<PathBuf>,
        max_depth: Option<usize>,
    ) -> Result<Self> {
        let base_dir = base_dir
            .or_else(|| std::env::current_dir().ok())
            .ok_or(Error::GetCurrentWorkingDir)?;
        Ok(Self {
            pattern,
            base_dir,
            max_depth: max_depth.unwrap_or(DEFAULT_MAX_DEPTH),
        })
    }

    pub fn glob_paths(&self) -> Result<Vec<PathBuf>> {
        paths(&self.pattern, &self.base_dir, Some(self.max_depth))
    }
}

/// Returns a GlobWalker instance with base path set to `base_path` and pattern to `pattern`. If
/// max_depth is specified the GlobWalker will have it's max depth set to its value, otherwise max
/// depth will be [DEFAULT_MAX_DEPTH](DEFAULT_MAX_DEPTH).
pub fn walker<S>(dir: S, pattern: S, max_depth: Option<usize>) -> Result<GlobWalker>
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

pub fn paths<P>(pattern: &str, base_path: P, max_depth: Option<usize>) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let base_path = base_path.as_ref().to_string_lossy().to_string();

    Ok(walker(base_path.as_str(), pattern, max_depth)?
        .flatten()
        .map(|entry| entry.into_path())
        .collect())
}
