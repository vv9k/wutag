mod app;
mod client;
mod config;
mod fmt;
mod opt;

use clap::{CommandFactory, Parser};

use app::App;
use config::Config;
use opt::{Command, CompletionsOpts, Opts, Shell, APP_NAME};
use std::io;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Client(#[from] client::ClientError),
    #[error(transparent)]
    App(#[from] app::AppError),
    #[error("failed to glob pattern - {0}")]
    Glob(wutag_core::Error),
    #[error("invalid shell - {0}")]
    InvalidShell(String),
    #[error("invalid output format - {0}")]
    InvalidOutputFormat(String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn print_completions(opts: &CompletionsOpts) -> Result<()> {
    use clap_complete::{
        generate,
        shells::{Bash, Elvish, Fish, PowerShell, Zsh},
    };

    let mut app = Opts::command();

    match opts.shell {
        Shell::Bash => generate(Bash, &mut app, APP_NAME, &mut io::stdout()),
        Shell::Elvish => generate(Elvish, &mut app, APP_NAME, &mut io::stdout()),
        Shell::Fish => generate(Fish, &mut app, APP_NAME, &mut io::stdout()),
        Shell::PowerShell => generate(PowerShell, &mut app, APP_NAME, &mut io::stdout()),
        Shell::Zsh => generate(Zsh, &mut app, APP_NAME, &mut io::stdout()),
    }
    Ok(())
}

fn main() {
    let config = Config::load_default_location().unwrap_or_default();
    let opts = Opts::parse();

    if let Command::PrintCompletions(opts) = &opts.cmd {
        if let Err(e) = print_completions(opts) {
            eprintln!("Execution failed, reason: {}", e);
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    if let Err(e) = App::run(opts, config) {
        eprintln!("Execution failed, reason: {}", e);
    }
}
