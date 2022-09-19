use colored::{ColoredString, Colorize};
use std::path::Path;

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
