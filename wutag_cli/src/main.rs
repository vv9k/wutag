mod app;
mod client;
mod config;
mod opt;
mod util;

use clap::Parser;
use colored::Color::{self, *};

use app::App;
use config::Config;
use opt::Opts;

/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;

fn main() {
    let config = Config::load_default_location().unwrap_or_default();

    if let Err(e) = App::run(Opts::from_args(), config) {
        eprintln!("{}", e);
    }
}
