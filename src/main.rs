mod opt;
mod runner;
mod tags;
mod util;

use clap::Clap;
use colored::Color::{self, *};

use opt::WutagOpts;
use runner::WutagRunner;

/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;
/// Default colors used for tags
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

fn main() {
    match WutagRunner::new(WutagOpts::parse()) {
        Ok(wutag) => wutag.run(),
        Err(e) => eprintln!("{}", e.to_string()),
    }
}
