//! Utility functions used through this crate and by the main executable
use colored::{Color, ColoredString, Colorize};
use globwalk::{DirEntry, GlobWalker, GlobWalkerBuilder};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::tags::Tag;
use crate::{Error, DEFAULT_MAX_DEPTH};

pub fn fmt_err<E: Display>(err: E) -> String {
    format!(
        "{} {}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

pub fn fmt_ok<S: AsRef<str>>(msg: S) -> String {
    format!("{} {}", "OK".green().bold(), msg.as_ref().white().bold())
}

pub fn fmt_path<P: AsRef<Path>>(path: P) -> String {
    format!("{}", path.as_ref().display().to_string().bold().blue())
}

pub fn fmt_tag(tag: &Tag) -> ColoredString {
    tag.name().color(*tag.color()).bold()
}

/// Returns a GlobWalker instance with base path set to `base_path` and pattern to `pattern`. If
/// `recursive` is true the maximum depth is going to be [DEFAULT_MAX_DEPTH](DEFAULT_MAX_DEPTH)
/// otherwise `1` (only top level files).
pub fn glob_walker<S>(dir: S, pattern: S, recursive: bool) -> Result<GlobWalker, Error>
where
    S: AsRef<str>,
{
    let mut builder = GlobWalkerBuilder::new(dir.as_ref(), pattern.as_ref());

    if !recursive {
        builder = builder.max_depth(2);
    } else {
        builder = builder.max_depth(DEFAULT_MAX_DEPTH);
    }
    builder.build().map_err(Error::from)
}

/// Utility function that executes the function `f` on all directory entries that are Ok, by
/// default ignores all errors.
pub fn glob_ok<P, F>(pattern: &str, base_path: P, recursive: bool, f: F) -> Result<(), Error>
where
    P: AsRef<Path>,
    F: Fn(&DirEntry),
{
    let base_path = base_path.as_ref().to_string_lossy().to_string();

    for entry in glob_walker(base_path.as_str(), pattern, recursive)? {
        if let Ok(entry) = entry {
            f(&entry);
        }
    }

    Ok(())
}

/// Calculates a hash of an item that implements [Hash](std::hash::Hash)
pub fn calculate_hash<T>(item: &T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

/// Parses a [Color](colored::Color) from a foreground color string
pub fn color_from_fg_str(s: &str) -> Option<Color> {
    match s {
        "30" => Some(Color::Black),
        "31" => Some(Color::Red),
        "32" => Some(Color::Green),
        "33" => Some(Color::Yellow),
        "34" => Some(Color::Blue),
        "35" => Some(Color::Magenta),
        "36" => Some(Color::Cyan),
        "37" => Some(Color::White),
        "90" => Some(Color::BrightBlack),
        "91" => Some(Color::BrightRed),
        "92" => Some(Color::BrightGreen),
        "93" => Some(Color::BrightYellow),
        "94" => Some(Color::BrightBlue),
        "95" => Some(Color::BrightMagenta),
        "96" => Some(Color::BrightCyan),
        "97" => Some(Color::BrightWhite),
        color => {
            if color.starts_with("38;2;") {
                let mut it = s.split(';');
                it.next()?;
                it.next()?;
                Some(Color::TrueColor {
                    r: it.next()?.parse().ok()?,
                    g: it.next()?.parse().ok()?,
                    b: it.next()?.parse().ok()?,
                })
            } else {
                None
            }
        }
    }
}
