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
        #[clap(long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `base_path`.
        base_path: Option<PathBuf>,
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
        #[clap(long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `base_path`.
        base_path: Option<PathBuf>,
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
        #[clap(long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `base_path`.
        base_path: Option<PathBuf>,
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
        #[clap(long)]
        /// When this parameter is specified the program will match files against provided pattern
        /// at the location specified by `base_path`.
        base_path: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
    },
    /// Recursively searches down the filesystem, starting from the current directory, for files tagged
    /// with the given tags. By default only first level of the directory is scanned.
    Search {
        #[clap(required = true)]
        tags: Vec<String>,
        #[clap(short, long)]
        /// When this parameter is specified the program will look for files in the location
        /// specified by this parameter instead of current working directory.
        base_path: Option<PathBuf>,
        #[clap(long, short)]
        /// If this parameter is specified that path will be treated as a glob and will recursively
        /// iterate over all files.
        recursive: bool,
    },
}
