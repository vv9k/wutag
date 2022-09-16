//! Utility functions used through this crate and by the main executable
use colored::Color::*;
pub use colored::{control, Color};

use crate::{Error, Result};

pub const DEFAULT_COLORS: &[Color] = &[
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    White,
    Magenta,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
];

/// Parses a [Color](colored::Color) from a foreground color string
pub fn color_from_fg_str(s: &str) -> Option<Color> {
    match s {
        "30" => Some(Black),
        "31" => Some(Red),
        "32" => Some(Green),
        "33" => Some(Yellow),
        "34" => Some(Blue),
        "35" => Some(Magenta),
        "36" => Some(Cyan),
        "37" => Some(White),
        "90" => Some(BrightBlack),
        "91" => Some(BrightRed),
        "92" => Some(BrightGreen),
        "93" => Some(BrightYellow),
        "94" => Some(BrightBlue),
        "95" => Some(BrightMagenta),
        "96" => Some(BrightCyan),
        "97" => Some(BrightWhite),
        color => {
            if color.starts_with("38;2;") {
                let mut it = s.split(';');
                it.next()?;
                it.next()?;
                Some(TrueColor {
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

const fn hex_chars_to_u8(ch: (u8, u8)) -> u8 {
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
    } else if let Some(c) = color.strip_prefix('#') {
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
