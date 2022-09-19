use anyhow::{Context, Error as AnyError, Result};
use clap::IntoApp;
use std::io;
use std::path::PathBuf;

use crate::client::Client;
use crate::config::Config;
use crate::opt::{
    ClearObject, ClearOpts, Command, CompletionsOpts, CpOpts, EditOpts, GetOpts, ListObject,
    ListOpts, Opts, RmOpts, SearchOpts, SetOpts, Shell, APP_NAME,
};
use crate::util::{fmt_path, fmt_tag, glob_paths};
use wutag_core::color::{self, parse_color, Color, DEFAULT_COLORS};
use wutag_core::tag::Tag;
use wutag_ipc::{default_socket, RequestResult, Response};

macro_rules! err {
    ($msg:tt) => {
        Err(anyhow::Error::msg(format!($msg)))
    };
}

pub struct App {
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
    pub colors: Vec<Color>,
    pub pretty: bool,
    pub client: Client,
}

impl App {
    pub fn run(opts: Opts, config: Config) -> Result<()> {
        let mut app = Self::new(&opts, config)?;
        app.run_command(opts.cmd);

        Ok(())
    }
    pub fn new(opts: &Opts, config: Config) -> Result<App> {
        let base_dir = if let Some(base_dir) = &opts.dir {
            base_dir.to_path_buf()
        } else {
            std::env::current_dir().context("failed to determine current working directory")?
        };

        let colors = if let Some(_colors) = config.colors {
            let mut colors = Vec::new();
            for color in _colors.iter().map(parse_color) {
                colors.push(color?);
            }
            colors
        } else {
            DEFAULT_COLORS.to_vec()
        };

        let client = Client::new(default_socket());

        if let Err(e) = client.ping() {
            return Err(AnyError::msg(format!("failed to connect to daemon, reason: {e}\nmake sure that the wutag daemon socket exists")));
        }

        Ok(App {
            base_dir,
            max_depth: if opts.max_depth.is_some() {
                opts.max_depth
            } else {
                config.max_depth
            },
            colors,
            pretty: opts.pretty || config.pretty_output,
            client,
        })
    }

    pub fn run_command(&mut self, cmd: Command) {
        if !self.pretty {
            color::control::SHOULD_COLORIZE.set_override(false);
        }
        let res = match cmd {
            Command::List(opts) => self.list(opts),
            Command::Set(opts) => self.set(opts),
            Command::Get(opts) => self.get(opts),
            Command::Rm(opts) => self.rm(opts),
            Command::Clear(opts) => self.clear(opts),
            Command::Search(opts) => self.search(opts),
            Command::Cp(opts) => self.cp(opts),
            Command::Edit(opts) => self.edit(opts),
            Command::PrintCompletions(opts) => self.print_completions(opts),
        }
        .context("failed executing command");

        if let Err(e) = res {
            eprintln!("Execution failed\n{e:?}");
            std::process::exit(1);
        }
    }

    fn clear_cache(&mut self) -> Result<()> {
        self.client.clear_cache().context("failed to clean cache")
    }

    fn get_paths(&self, glob: bool, paths: Vec<String>) -> Result<Vec<String>> {
        if glob {
            let paths = glob_paths(&paths[0], self.base_dir.clone(), self.max_depth)
                .context("failed to glob paths")?;
            Ok(paths
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect())
        } else {
            let mut parsed_paths = vec![];
            for path in paths {
                let path = PathBuf::from(path);
                match path.canonicalize() {
                    Ok(path) => parsed_paths.push(path.to_string_lossy().to_string()),
                    Err(e) => eprintln!(
                        "failed to canonicalize path `{}`, reason: {e:?}",
                        path.display()
                    ),
                }
            }
            Ok(parsed_paths)
        }
    }

    fn list(&self, opts: ListOpts) -> Result<()> {
        match opts.object {
            ListObject::Files { with_tags } => {
                let entries = match self
                    .client
                    .list_files(with_tags)
                    .context("failed to list entries")?
                {
                    Response::ListFiles(res) => match res {
                        RequestResult::Ok(entries) => entries,
                        RequestResult::Error(e) => {
                            return err!("Failed to list entries, reason: {e}");
                        }
                    },
                    response => {
                        return err!("Failed to list entries, reason: unexpected response from client {response:?}");
                    }
                };
                for (entry, tags) in entries {
                    print!("{}", fmt_path(entry.path()));
                    if let Some(mut tags) = tags {
                        tags.sort_unstable();
                        let tags = tags
                            .into_iter()
                            .map(|t| fmt_tag(&t).to_string())
                            .collect::<Vec<_>>()
                            .join(" ");

                        println!(": {}", tags);
                    } else {
                        println!();
                    }
                }
            }
            ListObject::Tags => {
                let mut tags = match self.client.list_tags().context("failed to list tags")? {
                    Response::ListTags(res) => match res {
                        RequestResult::Ok(tags) => tags,
                        RequestResult::Error(e) => {
                            return err!("Failed to list tags, reason: {e}");
                        }
                    },
                    response => {
                        return err!("Failed to list tags, reason: unexpected response from client {response:?}");
                    }
                };
                tags.sort_unstable();
                for tag in tags {
                    print!("{} ", fmt_tag(&tag));
                }
            }
        }
        Ok(())
    }

    fn set(&mut self, opts: SetOpts) -> Result<()> {
        if opts.paths.is_empty() {
            return err!("no entries to tag...");
        }
        let paths = self
            .get_paths(opts.glob, opts.paths)
            .context("failed to get a list of paths")?;

        let tags: Vec<_> = opts
            .tags
            .into_iter()
            .map(|t| Tag::random(t, &self.colors))
            .collect();

        match self
            .client
            .tag_files(paths, tags)
            .context("failed to tag files")?
        {
            Response::TagFiles(res) => {
                if let RequestResult::Error(e) = res {
                    eprintln!("Failed to tag some entries, reason: ");
                    for error in e {
                        eprintln!(" - {error}");
                    }
                    std::process::exit(1);
                }
            }
            response => {
                return err!(
                    "Failed to set tags on entries, reason: unexpected response from client {response:?}"
                );
            }
        };
        Ok(())
    }

    fn get(&mut self, opts: GetOpts) -> Result<()> {
        if opts.paths.is_empty() {
            return err!("no entries to check...");
        }

        let paths = self
            .get_paths(opts.glob, opts.paths)
            .context("failed to get a list of paths")?;

        let entries = match self
            .client
            .inspect_files(paths)
            .context("failed to inspect files")?
        {
            Response::InspectFiles(res) => match res {
                RequestResult::Ok(entries) => entries,
                RequestResult::Error(e) => {
                    return err!("Failed to inspect entries, reason: {e}");
                }
            },
            response => {
                return err!(
                    "Failed to inspect entries, reason: unexpected response from client {response:?}"
                );
            }
        };

        for (entry, mut tags) in entries {
            tags.sort_unstable();
            print!("{}:", fmt_path(entry.path()));
            for tag in &tags {
                print!(" {}", fmt_tag(tag))
            }
        }
        Ok(())
    }

    fn rm(&mut self, opts: RmOpts) -> Result<()> {
        if opts.paths.is_empty() {
            return err!("no entries to remove tags from...");
        }
        let paths = self
            .get_paths(opts.glob, opts.paths)
            .context("failed to get a list of paths")?;
        let tags: Vec<_> = opts
            .tags
            .into_iter()
            .map(|t| Tag::random(t, &self.colors))
            .collect();

        match self
            .client
            .untag_files(paths, tags)
            .context("faield to untag files")?
        {
            Response::UntagFiles(res) => {
                if let RequestResult::Error(e) = res {
                    let e: Vec<_> = e
                        .into_iter()
                        .filter(|e| !e.contains("doesn't exist"))
                        .collect();
                    if e.is_empty() {
                        return Ok(());
                    }
                    eprintln!("Failed to untag some entries, reason: ");
                    for error in e {
                        eprintln!(" - {error}");
                    }
                    std::process::exit(1);
                }
            }
            response => {
                return err!(
                    "Failed to remove tags from entries, reason: unexpected response from client {response:?}"
                );
            }
        };
        Ok(())
    }

    fn clear(&mut self, opts: ClearOpts) -> Result<()> {
        match opts.object {
            ClearObject::Files { paths, glob } => {
                if paths.is_empty() {
                    return err!("no entries to remove tags from...");
                }
                let paths = self
                    .get_paths(glob, paths)
                    .context("failed to get a list of paths")?;

                match self
                    .client
                    .clear_files(paths)
                    .context("failed to clear entries")?
                {
                    Response::ClearFiles(res) => {
                        if let RequestResult::Error(e) = res {
                            eprintln!("Failed to clear tags of some entries, reason: ");
                            for error in e {
                                eprintln!(" - {error}");
                            }
                            std::process::exit(1);
                        }
                    }
                    response => {
                        return err!(
                    "Failed to clear tags from entries, reason: unexpected response from client {response:?}"
                );
                    }
                };
            }
            ClearObject::Tags { names } => {
                if names.is_empty() {
                    return err!("no tags to clear...");
                }
                match self
                    .client
                    .clear_tags(names)
                    .context("failed to clear tags")?
                {
                    Response::ClearTags(res) => {
                        if let RequestResult::Error(e) = res {
                            eprintln!("Failed to clear tags, reason: ");
                            for error in e {
                                eprintln!(" - {error}");
                            }
                            std::process::exit(1);
                        }
                    }
                    response => {
                        return err!(
                    "Failed to clear tags, reason: unexpected response from client {response:?}"
                );
                    }
                };
            }
            ClearObject::Cache => self.clear_cache()?,
        }
        Ok(())
    }

    fn search(&self, opts: SearchOpts) -> Result<()> {
        let entries = match self
            .client
            .search(opts.tags, opts.any)
            .context("failed to search")?
        {
            Response::Search(res) => match res {
                RequestResult::Ok(entries) => entries,
                RequestResult::Error(e) => {
                    return err!("Failed to search entries with tags, reason: {e}");
                }
            },
            response => {
                return err!(
                    "Failed to search entries with tags, reason: unexpected response from client {response:?}"
                );
            }
        };
        for entry in entries {
            println!("{}", fmt_path(entry.path()));
        }
        Ok(())
    }

    fn cp(&mut self, opts: CpOpts) -> Result<()> {
        let paths = self
            .get_paths(opts.glob, opts.paths)
            .context("failed to get a list of paths")?;

        match self
            .client
            .copy_tags(opts.input_path, paths)
            .context("failed to copy tags")?
        {
            Response::CopyTags(res) => {
                if let RequestResult::Error(e) = res {
                    eprintln!("Failed to copy tags, reason: ");
                    for error in e {
                        eprintln!(" - {error}");
                    }
                    std::process::exit(1);
                }
            }
            response => {
                return err!(
                    "Failed to copy tags, reason: unexpected response from client {response:?}"
                );
            }
        };
        Ok(())
    }

    fn edit(&mut self, opts: EditOpts) -> Result<()> {
        let c = match parse_color(&opts.color) {
            Ok(color) => color,
            Err(e) => {
                return err!("{e}");
            }
        };

        match self
            .client
            .edit_tag(opts.tag, c)
            .context("failed to edit tag")?
        {
            Response::EditTag(res) => {
                if let RequestResult::Error(e) = res {
                    return err!("Failed to edit tag, reason: {e}");
                }
            }
            response => {
                return err!(
                    "Failed to copy tags, reason: unexpected response from client {response:?}"
                );
            }
        };
        Ok(())
    }

    fn print_completions(&self, opts: CompletionsOpts) -> Result<()> {
        use clap_complete::{
            generate,
            shells::{Bash, Elvish, Fish, PowerShell, Zsh},
        };

        let mut app = Opts::into_app();

        match opts.shell {
            Shell::Bash => generate(Bash, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Elvish => generate(Elvish, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Fish => generate(Fish, &mut app, APP_NAME, &mut io::stdout()),
            Shell::PowerShell => generate(PowerShell, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Zsh => generate(Zsh, &mut app, APP_NAME, &mut io::stdout()),
        }
        Ok(())
    }
}
