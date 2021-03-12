//! Options used by the main executable
use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct WutagOpts {
    #[clap(short, long)]
    /// When this parameter is specified the program will look for files starting from provided
    /// path, otherwise defaults to current directory.
    pub dir: Option<PathBuf>,
    #[clap(long, short)]
    /// Increase maximum recursion depth of filesystem traversal to 512. Default is 2. What this
    /// means is by deafult all subcommands that take a pattern as input will match files only 2
    /// levels deep starting from the base directory which is current working directory if `dir` is
    /// not specified.
    pub recursive: bool,
    /// If passed the output won't be colored
    #[clap(long, short)]
    pub no_color: bool,
    #[clap(subcommand)]
    pub cmd: WutagCmd,
}

#[derive(Clap)]
pub struct ListOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    #[clap(long)]
    /// Whether to show files with no tags
    pub show_missing: bool,
}

#[derive(Clap)]
pub struct SetOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    #[clap(required = true)]
    pub tags: Vec<String>,
}

#[derive(Clap)]
pub struct RmOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    pub tags: Vec<String>,
}

#[derive(Clap)]
pub struct ClearOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
}
#[derive(Clap)]
pub struct SearchOpts {
    #[clap(required = true)]
    pub tags: Vec<String>,
    #[clap(long)]
    /// If provided output will be raw so that it can be easily piped to other commands
    pub raw: bool,
}

#[derive(Clap)]
pub struct CpOpts {
    /// Path to the file from which to copy tags from
    pub input_path: PathBuf,
    /// A glob pattern like '*.png'.
    pub pattern: String,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files that match the provided pattern in the current working
    /// directory. By default only first level of the directory is scanned.
    List(ListOpts),
    /// Tags the files located at the given `path` with the set of `tags`. By default only first level of the directory is processed.
    Set(SetOpts),
    /// Removes the specified tags of the files that match the provided pattern in the current
    /// working directory. By default only first level of the directory is processed.
    Rm(RmOpts),
    /// Clears all tags of the files that match the provided pattern in the current working directory.
    /// By default only first level of the directory is processed.
    Clear(ClearOpts),
    /// Searches for files that have all of the provided `tags` in the current directory.
    Search(SearchOpts),
    /// Copies tags from the specified file to files that match a pattern
    Cp(CpOpts),
}
