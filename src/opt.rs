use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct WutagOpts {
    #[clap(subcommand)]
    pub cmd: WutagCmd,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files located at the given path.
    List {
        /// A normal path or a glob like `'*.png'`.
        path: String,
        #[clap(long)]
        /// Whether to show files with no tags
        show_missing: bool,
    },
    /// Tags the files located at the given `path` with the set of `tags`.
    Set {
        #[clap(takes_value = true, required = true)]
        /// A normal path or a glob like `'*.png'`.
        path: String,
        tags: Vec<String>,
    },
    /// Removes the specified tags of the files located at the give path.
    Rm {
        #[clap(takes_value = true, required = true)]
        /// A normal path or a glob like `'*.png'`.
        path: String,
        tags: Vec<String>,
    },
    /// Clears all tags of the files located at the given paths.
    Clear {
        #[clap(required = true)]
        /// A normal path or a glob like `'*.png'`.
        path: String,
    },
    /// Recursively searches down the filesystem, starting from the current directory, for files tagged
    /// with the given tags.
    Search {
        #[clap(required = true)]
        tags: Vec<String>,
        #[clap(short, long)]
        /// A normal path or a glob like `'*.png'`
        path: Option<String>,
    },
}
