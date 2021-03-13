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
    #[clap(short, long)]
    /// If specified output and errors will be displayed
    pub verbose: bool,
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
pub struct EditOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    /// The tag to edit
    pub tag: String,
    #[clap(long, short)]
    /// Set the color of the tag to the specified color. Accepted values are hex colors like
    /// `0x000000` or `#1F1F1F` or just plain `ff000a`. The colors are case insensitive meaning
    /// `1f1f1f` is equivalent to `1F1F1F`.
    pub color: String,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files that match the provided pattern.
    List(ListOpts),
    /// Tags the files that match the given pattern with specified tags.
    Set(SetOpts),
    /// Removes the specified tags of the files that match the provided pattern.
    Rm(RmOpts),
    /// Clears all tags of the files that match the provided pattern.
    Clear(ClearOpts),
    /// Searches for files that have all of the provided `tags`.
    Search(SearchOpts),
    /// Copies tags from the specified file to files that match a pattern.
    Cp(CpOpts),
    /// Edits the tag of files that match the provided pattern.
    Edit(EditOpts),
}
