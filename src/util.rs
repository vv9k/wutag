//! Utility functions used through this crate and by the main executable
use colored::{Color, ColoredString, Colorize};
use globwalk::{DirEntry, GlobWalker, GlobWalkerBuilder};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::tags::Tag;
use crate::{Error, Result, DEFAULT_MAX_DEPTH};

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
/// `recursive` is true the maximum depth is going to be [DEFAULT_MAX_DEPTH](DEFAULT_MAX_DEPTH)
/// otherwise `1` (only top level files).
pub fn glob_walker<S>(dir: S, pattern: S, recursive: bool) -> Result<GlobWalker>
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
pub fn glob_ok<P, F>(pattern: &str, base_path: P, recursive: bool, f: F) -> Result<()>
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

const fn hex_val(ch: u8) -> u8 {
    match ch {
        b'0'..=b'9' => ch - 48,
        b'A'..=b'F' => ch - 55,
        b'a'..=b'f' => ch - 87,
        _ => 0,
    }
}

fn hex_chars_to_u8(ch: (u8, u8)) -> u8 {
    let mut result = 0;
    result |= hex_val(ch.0);
    result <<= 4;
    result |= hex_val(ch.1);
    result
}

fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let mut bytes = color.as_bytes().chunks(2);

    Some((
        bytes.next().map(|arr| hex_chars_to_u8((arr[0], arr[1])))?,
        bytes.next().map(|arr| hex_chars_to_u8((arr[0], arr[1])))?,
        bytes.next().map(|arr| hex_chars_to_u8((arr[0], arr[1])))?,
    ))
}

/// Parses a [Color](colored::Color) from a String. If the provided string starts with
/// `0x` or `#` or without any prefix the color will be treated as hex color notation so any colors like `0x1f1f1f` or
/// `#ABBA12` or `121212` are valid.
pub fn parse_color<S: AsRef<str>>(color: S) -> Result<Color> {
    let color = color.as_ref();
    macro_rules! if_6 {
        ($c:ident) => {
            if $c.len() == 6 {
                Some($c)
            } else {
                None
            }
        };
    }

    let result = if let Some(c) = color.strip_prefix("0x") {
        if_6!(c)
    } else if let Some(c) = color.strip_prefix("#") {
        if_6!(c)
    } else {
        if_6!(color)
    };

    if let Some(color) = result {
        // hex
        if let Some((r, g, b)) = parse_hex(color) {
            return Ok(Color::TrueColor { r, g, b });
        }
    }
    Err(Error::InvalidColor(color.to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_color;
    use colored::Color::*;
    #[test]
    fn parses_colors() {
        assert_eq!(
            parse_color("0xffffff").unwrap(),
            TrueColor {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            parse_color("#ffffff").unwrap(),
            TrueColor {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            parse_color("0ff00f").unwrap(),
            TrueColor {
                r: 15,
                g: 240,
                b: 15
            }
        );
    }
    #[test]
    fn errors_on_invalid_colors() {
        assert!(parse_color("0ff00").is_err());
        assert!(parse_color("0x12345").is_err());
        assert!(parse_color("#53241").is_err());
        assert!(parse_color("1234567").is_err());
        assert!(parse_color("#1234567").is_err());
        assert!(parse_color("0x1234567").is_err());
    }
}
