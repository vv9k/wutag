use wutag_core::color::{ColoredString, Colorize};
use wutag_core::tag::Tag;

use std::path::Path;

pub fn path<P: AsRef<Path>>(path: P) -> ColoredString {
    path.as_ref().display().to_string().bold().blue()
}

pub fn tag(tag: &Tag) -> ColoredString {
    if tag.name().chars().any(|c| c.is_ascii_whitespace()) {
        format!("\"{}\"", tag.name()).color(*tag.color()).bold()
    } else {
        tag.name().color(*tag.color()).bold()
    }
}
