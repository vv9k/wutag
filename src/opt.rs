use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct WutagOpts {
    #[clap(subcommand)]
    pub cmd: WutagCmd,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files located at the given paths.
    List {
        #[clap(required = true)]
        paths: Vec<PathBuf>,
        #[clap(short, long)]
        /// If enabled treats all provided paths as directories and recursively lists tags of all
        /// files in those directories and all subdirectories. This flag has higher precedence than
        /// `dirs` flag.
        recursive: bool,
        #[clap(long)]
        /// Whether to show files with no tags
        show_missing: bool,
    },
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
    /// Clears all tags of the files located at the given paths.
    Clear {
        #[clap(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Recursively searches down the filesystem, starting from the current directory, for files tagged
    /// with the given tags.
    Search {
        #[clap(required = true)]
        tags: Vec<String>,
        #[clap(short, long)]
        /// Change starting directory from current directory
        path: Option<PathBuf>,
    },
}
