//! Options used by the main executable
use std::{path::PathBuf, str::FromStr};

use anyhow::Error;
use clap::Parser;

pub const APP_NAME: &str = "wutag";
pub const APP_VERSION: &str = "0.5.0-dev";
pub const APP_AUTHOR: &str = "Wojciech Kępka <wojciech@wkepka.dev>";
pub const APP_ABOUT: &str = "Tool to tag and manage tags of files.";

#[derive(Parser)]
#[clap(
    version = APP_VERSION,
    author = APP_AUTHOR,
    about = APP_ABOUT,
)]
pub struct Opts {
    #[clap(short, long)]
    /// When this parameter is specified the program will look for files starting from provided
    /// path, otherwise defaults to current directory. Only applies to subcommands that take a
    /// pattern as a positional argument.
    pub dir: Option<PathBuf>,
    #[clap(long, short)]
    /// If provided increase maximum recursion depth of filesystem traversal to specified value,
    /// otherwise default depth is 2. Only applies to subcommands that take a pattern as a
    /// positional argument.
    pub max_depth: Option<usize>,
    /// Make the output pretty (add color and reorder things). This is not recommended when using
    /// wutag in scripts.
    #[clap(long, short)]
    pub pretty: bool,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum ListObject {
    Tags,
    Files {
        #[clap(long, short = 't')]
        /// Should the tags of the entry be display.
        with_tags: bool,
    },
}

#[derive(Parser)]
pub struct ListOpts {
    #[clap(subcommand)]
    /// The object to list. Valid values are: `tags`, `files`.
    pub object: ListObject,
}

#[derive(Parser)]
pub struct SetOpts {
    /// A list of entries to tag
    pub paths: Vec<String>,
    #[clap(short, long)]
    /// Treat the first path as a glob pattern
    pub glob: bool,
    #[clap(required = true, last = true)]
    /// List of tags to tag the entries with
    pub tags: Vec<String>,
}

#[derive(Parser)]
pub struct GetOpts {
    /// A list of entries to retrieve tags from
    pub paths: Vec<String>,
    #[clap(short, long)]
    /// Treat the first path as a glob pattern
    pub glob: bool,
}

#[derive(Parser)]
pub struct RmOpts {
    /// A list of entries to tag
    pub paths: Vec<String>,
    #[clap(short, long)]
    /// Treat the first path as a glob pattern
    pub glob: bool,
    #[clap(required = true, last = true)]
    pub tags: Vec<String>,
}

#[derive(Parser)]
pub enum ClearObject {
    /// Remove tags completely
    Tags {
        /// The names of the tags to clear from all entries
        names: Vec<String>,
    },
    /// Remove all tags from specified files
    Files {
        /// A list of entries to tag
        paths: Vec<String>,
        #[clap(short, long)]
        /// Treat the first path as a glob pattern
        glob: bool,
    },
    Cache,
}

#[derive(Parser)]
pub struct ClearOpts {
    #[clap(subcommand)]
    /// The object to list. Valid values are: `tags`, `files`.
    pub object: ClearObject,
}

#[derive(Parser)]
pub struct SearchOpts {
    #[clap(required = true)]
    pub tags: Vec<String>,
    #[clap(long, short)]
    /// If set to 'true' all entries containing any of provided tags will be returned
    pub any: bool,
}

#[derive(Parser)]
pub struct CpOpts {
    #[clap(short, long)]
    /// Treat the first path as a glob pattern
    pub glob: bool,
    /// Path to the file from which to copy tags from
    pub input_path: PathBuf,
    #[clap(required = true, last = true)]
    /// A list of entries to tag
    pub paths: Vec<String>,
}

#[derive(Parser)]
pub struct EditOpts {
    /// The tag to edit
    pub tag: String,
    #[clap(long, short)]
    /// Set the color of the tag to the specified color. Accepted values are hex colors like
    /// `0x000000` or `#1F1F1F` or just plain `ff000a`. The colors are case insensitive meaning
    /// `1f1f1f` is equivalent to `1F1F1F`.
    pub color: String,
}

#[derive(Parser)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl FromStr for Shell {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "bash" => Ok(Shell::Bash),
            "elvish" => Ok(Shell::Elvish),
            "fish" => Ok(Shell::Fish),
            "powershell" => Ok(Shell::PowerShell),
            "zsh" => Ok(Shell::Zsh),
            _ => Err(Error::msg(format!("invalid shell `{}`", s))),
        }
    }
}

#[derive(Parser)]
pub struct CompletionsOpts {
    /// A shell for which to print completions. Available shells are: bash, elvish, fish,
    /// powershell, zsh
    pub shell: Shell,
}

#[derive(Parser)]
pub enum Command {
    /// Lists all available tags or files.
    List(ListOpts),
    /// Tags the files that match the given pattern with specified tags.
    Set(SetOpts),
    /// Retrieve tags of files
    Get(GetOpts),
    /// Removes the specified tags of the files that match the provided pattern.
    Rm(RmOpts),
    /// Clears all tags of the files that match the provided pattern.
    Clear(ClearOpts),
    /// Searches for files that have all of the provided 'tags'.
    Search(SearchOpts),
    /// Copies tags from the specified file to files that match a pattern.
    Cp(CpOpts),
    /// Edits a tag.
    Edit(EditOpts),
    /// Prints completions for the specified shell to stdout.
    PrintCompletions(CompletionsOpts),
}
