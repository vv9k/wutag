//! Options used by the main executable
use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct WutagOpts {
    #[clap(subcommand)]
    pub cmd: WutagCmd,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files that match the provided pattern in the current working
    /// directory. By default only first level of the directory is scanned.
    List {
        /// A glob pattern like '*.png'.
        pattern: String,
        #[clap(short, long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `dir`.
        dir: Option<PathBuf>,
        #[clap(long)]
        /// Whether to show files with no tags
        show_missing: bool,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
    },
    /// Tags the files located at the given `path` with the set of `tags`. By default only first level of the directory is processed.
    Set {
        /// A glob pattern like '*.png'.
        pattern: String,
        #[clap(short, long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `dir`.
        dir: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
        #[clap(required = true)]
        tags: Vec<String>,
    },
    /// Removes the specified tags of the files that match the provided pattern in the current
    /// working directory. By default only first level of the directory is processed.
    Rm {
        /// A glob pattern like '*.png'.
        pattern: String,
        #[clap(short, long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `dir`.
        dir: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
        tags: Vec<String>,
    },
    /// Clears all tags of the files that match the provided pattern in the current working directory.
    /// By default only first level of the directory is processed.
    Clear {
        /// A glob pattern like '*.png'.
        pattern: String,
        #[clap(short, long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `dir`.
        dir: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
    },
    /// Searches for files that have all of the provided `tags` in the current directory.
    Search {
        #[clap(required = true)]
        tags: Vec<String>,
        #[clap(short, long)]
        /// When this parameter is specified the program will look for files in the location
        /// specified by this parameter instead of the current working directory.
        dir: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified recursively look for files in all subdirectories.
        recursive: bool,
        #[clap(long)]
        /// If provided output will be raw so that it can be easily piped to other commands
        raw: bool,
    },
    /// Copies tags from the specified file to files that match a pattern
    Cp {
        /// Path to the file from which to copy tags from
        input_path: PathBuf,
        /// A glob pattern like '*.png'.
        pattern: String,
        #[clap(short, long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `dir`.
        dir: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
    },
}
