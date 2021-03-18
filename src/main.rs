mod opt;
mod runner;
mod tags;
mod util;

use clap::Clap;

use opt::WutagOpts;
use runner::WutagRunner;

/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;

fn main() {
    match WutagRunner::new(WutagOpts::parse()) {
        Ok(wutag) => wutag.run(),
        Err(e) => eprintln!("{}", e.to_string()),
    }
}
