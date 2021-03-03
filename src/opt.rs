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
    List { path: PathBuf },
    /// Tags the files located at the given paths with the set of tags.
    Set {
        #[clap(takes_value = true, required = true)]
        paths: Vec<PathBuf>,
        #[clap(last = true)]
        tags: Vec<String>,
    },
    /// Removes the specified tags of the files located at the given paths.
    Rm {
        #[clap(takes_value = true, required = true)]
        paths: Vec<PathBuf>,
        #[clap(last = true)]
        tags: Vec<String>,
    },
    /// Clears all tags of the file located at the given path
    Clear { path: PathBuf },
    /// Recursively searches down the filesystem for files tagged with the given tag
    Search { tag: String },
}
