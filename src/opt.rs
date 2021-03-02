use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct RutagOpts {
    #[clap(subcommand)]
    pub cmd: RutagCmd,
}

#[derive(Clap)]
pub enum RutagCmd {
    /// Lists all tags of the file located at the given path
    List {
        path: PathBuf,
        #[clap(short, long)]
        pretty: bool,
    },
    /// Sets a tag of the file located at at the given path
    Set { path: PathBuf, tag: String },
    /// Removes a tag of the file located at the given path
    Rm { path: PathBuf, tag: String },
    /// Clears all tags of the file located at the given path
    Clear { path: PathBuf },
}
