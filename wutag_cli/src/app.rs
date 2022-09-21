use clap::CommandFactory;
use std::io;
use std::path::PathBuf;

use crate::client::Client;
use crate::config::Config;
use crate::fmt;
use crate::opt::{
    ClearObject, ClearOpts, Command, CompletionsOpts, CpOpts, EditOpts, GetOpts, ListObject,
    ListOpts, Opts, OutputFormat, RmOpts, SearchOpts, SetOpts, Shell, APP_NAME,
};
use crate::{Error, Result};
use thiserror::Error as ThisError;
use wutag_core::color::{self, parse_color, Color, DEFAULT_COLORS};
use wutag_core::glob::Glob;
use wutag_core::tag::Tag;
use wutag_ipc::{default_socket, Response};

#[derive(Debug, ThisError)]
pub enum AppError {
    #[error("failed to determine current working directory - {0}")]
    GetCurrentWorkingDirectory(std::io::Error),
    #[error("failed to parse color - {0}")]
    ParseColor(wutag_core::Error),
    #[error("failed to list entries - {0}")]
    ListEntries(String),
    #[error("failed to inspect entries - {0}")]
    InspectEntries(String),
    #[error("failed to search - {0}")]
    Search(String),
    #[error("failed to list tags - {0}")]
    ListTags(String),
    #[error("failed to edit tag - {0}")]
    EditTag(String),
    #[error("failed to serialize output as yaml - {0}")]
    SerializeYamlOutput(serde_yaml::Error),
    #[error("failed to serialize output as json - {0}")]
    SerializeJsonOutput(serde_json::Error),
    #[error("failed to {action} - unexpected response from server {response:?}")]
    UnexpectedResponse { action: String, response: Response },
}

pub struct App {
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
    pub colors: Vec<Color>,
    pub pretty: bool,
    pub format: OutputFormat,
    pub client: Client,
}

impl App {
    pub fn run(opts: Opts, config: Config) -> Result<()> {
        let mut app = Self::new(&opts, config)?;
        app.run_command(opts.cmd)
    }
    pub fn new(opts: &Opts, config: Config) -> Result<App> {
        let base_dir = if let Some(base_dir) = &opts.dir {
            base_dir.to_path_buf()
        } else {
            std::env::current_dir().map_err(AppError::GetCurrentWorkingDirectory)?
        };

        let colors = if let Some(_colors) = config.colors {
            let mut colors = Vec::new();
            for color in _colors.iter().map(parse_color) {
                colors.push(color.map_err(AppError::ParseColor)?);
            }
            colors
        } else {
            DEFAULT_COLORS.to_vec()
        };

        let client = Client::new(default_socket());

        client.ping()?;

        Ok(App {
            base_dir,
            max_depth: if opts.max_depth.is_some() {
                opts.max_depth
            } else {
                config.max_depth
            },
            colors,
            pretty: opts.pretty || config.pretty_output,
            format: opts.output_format,
            client,
        })
    }

    pub fn run_command(&mut self, cmd: Command) -> Result<()> {
        if !self.pretty {
            color::control::SHOULD_COLORIZE.set_override(false);
        }
        match cmd {
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
    }

    fn print_serialized<T: serde::Serialize + std::fmt::Debug>(&self, it: T) -> Result<()> {
        let output = match self.format {
            OutputFormat::Json => {
                serde_json::to_string(&it).map_err(AppError::SerializeJsonOutput)?
            }
            OutputFormat::Yaml => {
                serde_yaml::to_string(&it).map_err(AppError::SerializeYamlOutput)?
            }
            OutputFormat::Default => format!("{it:?}"),
        };
        println!("{output}");
        Ok(())
    }

    fn clear_cache(&mut self) -> Result<()> {
        self.client.clear_cache()
    }

    fn list(&self, opts: ListOpts) -> Result<()> {
        match opts.object {
            ListObject::Files { with_tags } => {
                let entries = self.client.list_files(with_tags)?;
                match self.format {
                    OutputFormat::Json | OutputFormat::Yaml => {
                        let entries: std::collections::HashMap<_, _> = entries
                            .into_iter()
                            .map(|(e, tags)| {
                                (
                                    e.into_path_buf(),
                                    tags.into_iter().map(Tag::into_name).collect::<Vec<_>>(),
                                )
                            })
                            .collect();
                        self.print_serialized(entries)?;
                    }
                    OutputFormat::Default => {
                        for (entry, mut tags) in entries {
                            print!("{}", fmt::path(entry.path()));
                            tags.sort_unstable();
                            let tags = tags
                                .into_iter()
                                .map(|t| fmt::tag(&t).to_string())
                                .collect::<Vec<_>>()
                                .join(" ");

                            println!(": {}", tags);
                        }
                    }
                }
            }
            ListObject::Tags { with_files } => {
                let tags = self.client.list_tags(with_files)?;
                match self.format {
                    OutputFormat::Json | OutputFormat::Yaml => {
                        let tags: std::collections::HashMap<_, _> = tags
                            .into_iter()
                            .map(|(t, e)| {
                                (
                                    t.into_name(),
                                    e.into_iter().map(|e| e.into_path_buf()).collect::<Vec<_>>(),
                                )
                            })
                            .collect();
                        self.print_serialized(tags)?;
                    }
                    OutputFormat::Default => {
                        if with_files {
                            for (tag, entries) in tags {
                                println!("{}:", fmt::tag(&tag));
                                for entry in entries {
                                    println!("\t{}", fmt::path(entry.path()));
                                }
                            }
                        } else {
                            let mut tags: Vec<_> = tags.into_iter().map(|(t, _)| t).collect();
                            tags.sort_unstable();
                            for tag in tags {
                                print!("{} ", fmt::tag(&tag));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn set(&mut self, opts: SetOpts) -> Result<()> {
        let tags: Vec<_> = opts
            .tags
            .into_iter()
            .map(|t| Tag::random(t, &self.colors))
            .collect();

        if opts.glob {
            let glob = self.glob(&opts.paths[0])?;
            self.client
                .tag_files_pattern(glob, tags)
                .map_err(Error::from)
                .map(|_| ())
        } else {
            self.client
                .tag_files(opts.paths, tags)
                .map_err(Error::from)
                .map(|_| ())
        }
    }

    fn get(&mut self, opts: GetOpts) -> Result<()> {
        let entries = if opts.glob {
            let glob = self.glob(&opts.paths[0])?;
            self.client.inspect_files_pattern(glob)?
        } else {
            self.client.inspect_files(opts.paths)?
        };

        match self.format {
            OutputFormat::Json | OutputFormat::Yaml => {
                let entries: std::collections::HashMap<_, _> = entries
                    .into_iter()
                    .map(|(e, tags)| (e.into_path_buf(), tags))
                    .collect();
                self.print_serialized(entries)?;
            }
            OutputFormat::Default => {
                for (entry, mut tags) in entries {
                    tags.sort_unstable();
                    print!("{}:", fmt::path(entry.path()));
                    for tag in &tags {
                        print!(" {}", fmt::tag(tag))
                    }
                }
            }
        }
        Ok(())
    }

    fn rm(&mut self, opts: RmOpts) -> Result<()> {
        let tags: Vec<_> = opts
            .tags
            .into_iter()
            .map(|t| Tag::random(t, &self.colors))
            .collect();

        if opts.glob {
            let glob = self.glob(&opts.paths[0])?;
            self.client
                .untag_files_pattern(glob, tags)
                .map_err(Error::from)
                .map(|_| ())
        } else {
            self.client
                .untag_files(opts.paths, tags)
                .map_err(Error::from)
                .map(|_| ())
        }
    }

    fn clear(&mut self, opts: ClearOpts) -> Result<()> {
        match opts.object {
            ClearObject::Files { paths, glob } => {
                if glob {
                    let glob = self.glob(&paths[0])?;
                    self.client.clear_files_pattern(glob)?;
                } else {
                    self.client.clear_files(paths)?;
                }
            }
            ClearObject::Tags { names } => {
                self.client.clear_tags(names)?;
            }
            ClearObject::Cache => self.clear_cache()?,
        }
        Ok(())
    }

    fn search(&self, opts: SearchOpts) -> Result<()> {
        let entries = self.client.search(opts.tags, opts.any)?;
        match self.format {
            OutputFormat::Json | OutputFormat::Yaml => {
                let entries: Vec<_> = entries.into_iter().map(|e| e.into_path_buf()).collect();
                self.print_serialized(entries)?;
            }
            OutputFormat::Default => {
                for entry in entries {
                    println!("{}", fmt::path(entry.path()));
                }
            }
        }
        Ok(())
    }

    fn cp(&mut self, opts: CpOpts) -> Result<()> {
        if opts.glob {
            let glob = self.glob(&opts.paths[0])?;
            self.client
                .copy_tags_pattern(opts.input_path, glob)
                .map_err(Error::from)
                .map(|_| ())
        } else {
            self.client
                .copy_tags(opts.input_path, opts.paths)
                .map_err(Error::from)
                .map(|_| ())
        }
    }

    fn edit(&mut self, opts: EditOpts) -> Result<()> {
        let c = parse_color(&opts.color).map_err(AppError::ParseColor)?;

        self.client
            .edit_tag(opts.tag, c)
            .map_err(Error::from)
            .map(|_| ())
    }

    fn print_completions(&self, opts: CompletionsOpts) -> Result<()> {
        use clap_complete::{
            generate,
            shells::{Bash, Elvish, Fish, PowerShell, Zsh},
        };

        let mut app = Opts::command();

        match opts.shell {
            Shell::Bash => generate(Bash, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Elvish => generate(Elvish, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Fish => generate(Fish, &mut app, APP_NAME, &mut io::stdout()),
            Shell::PowerShell => generate(PowerShell, &mut app, APP_NAME, &mut io::stdout()),
            Shell::Zsh => generate(Zsh, &mut app, APP_NAME, &mut io::stdout()),
        }
        Ok(())
    }

    fn glob(&self, pattern: impl Into<String>) -> Result<Glob> {
        Glob::new(pattern.into(), Some(self.base_dir.clone()), self.max_depth).map_err(Error::Glob)
    }
}
