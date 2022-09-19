mod app;
mod client;
mod config;
mod opt;
mod util;

use clap::Parser;

use app::App;
use config::Config;
use opt::Opts;
use thiserror::Error as ThisError;

/// Default max depth passed to [GlobWalker](globwalker::GlobWalker)
pub const DEFAULT_MAX_DEPTH: usize = 2;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Client(#[from] client::ClientError),
    #[error(transparent)]
    App(#[from] app::AppError),
    #[error("failed to glob pattern - {0}")]
    Glob(#[from] globwalk::GlobError),
    #[error("invalid shell - {0}")]
    InvalidShell(String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    let config = Config::load_default_location().unwrap_or_default();

    if let Err(e) = App::run(Opts::from_args(), config) {
        eprintln!("Execution failed, reason: {}", e);
    }
}
