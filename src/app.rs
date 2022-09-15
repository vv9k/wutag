use anyhow::{Context, Result};
use clap::IntoApp;
use colored::{Color, Colorize};
use globwalk::DirEntry;
use std::io;
use std::path::PathBuf;

use crate::config::Config;
use crate::opt::{
    ClearOpts, Command, CompletionsOpts, CpOpts, EditOpts, ListObject, ListOpts, Opts, RmOpts,
    SearchOpts, SetOpts, Shell, APP_NAME,
};
use crate::registry::{EntryData, TagRegistry};
use crate::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok};
use crate::DEFAULT_COLORS;
use wutag_core::color::parse_color;
use wutag_core::tag::{list_tags, DirEntryExt, Tag};
use wutag_core::Error;

pub struct App {
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
    pub colors: Vec<Color>,
    pub pretty: bool,
    pub registry: TagRegistry,
}

macro_rules! err {
    ($err:ident, $entry:ident) => {
        err!("", $err, $entry);
    };
    ($prefix:expr, $err:ident, $entry:ident) => {{
        let err = fmt_err($err);
        eprintln!("{}{}", $prefix, err);
    }};
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

        let cache_dir = dirs::cache_dir().context("failed to determine cache directory")?;
        let state_file = cache_dir.join("wutag.registry");

        let registry =
            TagRegistry::load(&state_file).unwrap_or_else(|_| TagRegistry::new(&state_file));

        Ok(App {
            base_dir,
            max_depth: if opts.max_depth.is_some() {
                opts.max_depth
            } else {
                config.max_depth
            },
            colors,
            pretty: opts.pretty || config.pretty_output,
            registry,
        })
    }

    fn save_registry(&mut self) {
        if let Err(e) = self.registry.save() {
            eprintln!("failed to save registry - {}", e);
        }
    }

    pub fn run_command(&mut self, cmd: Command) {
        if !self.pretty {
            colored::control::SHOULD_COLORIZE.set_override(false);
        }
        match cmd {
            Command::List(ref opts) => self.list(opts),
            Command::Set(opts) => self.set(&opts),
            Command::Rm(ref opts) => self.rm(opts),
            Command::Clear(ref opts) => self.clear(opts),
            Command::Search(ref opts) => self.search(opts),
            Command::Cp(ref opts) => self.cp(opts),
            Command::Edit(ref opts) => self.edit(opts),
            Command::PrintCompletions(ref opts) => self.print_completions(opts),
            Command::CleanCache => self.clean_cache(),
            Command::UpdateRegistry => self.update_registry(),
        }
    }

    fn clean_cache(&mut self) {
        self.registry.clear();
        if let Err(e) = self.registry.save() {
            println!("{:?}", e);
        }
    }

    fn update_registry(&mut self) {
        let mut entries_to_remove = vec![];
        println!("Entries removed:");
        for (id, file) in self.registry.list_entries_and_ids() {
            if !file.path().exists() {
                println!(" - {}", file.path().display());
                entries_to_remove.push(*id);
            }
        }
        println!("Total: {}", entries_to_remove.len());
        entries_to_remove.into_iter().for_each(|entry| {
            self.registry.clear_entry(entry);
        });
        self.save_registry();
    }

    fn list(&self, opts: &ListOpts) {
        match opts.object {
            ListObject::Files { with_tags } => {
                for (id, file) in self.registry.list_entries_and_ids() {
                    if self.pretty {
                        print!("{}", fmt_path(file.path()));
                    } else {
                        print!("{}", file.path().display());
                    }
                    if with_tags {
                        let tags = self
                            .registry
                            .list_entry_tags(*id)
                            .unwrap_or_default()
                            .iter()
                            .map(|t| {
                                if self.pretty {
                                    fmt_tag(t).to_string()
                                } else {
                                    t.name().to_owned()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");

                        println!(": {}", tags);
                    } else {
                        println!();
                    }
                }
            }
            ListObject::Tags => {
                for tag in self.registry.list_tags() {
                    if self.pretty {
                        print!("{}\t", fmt_tag(tag));
                    } else {
                        print!("{}\t", tag);
                    }
                }
            }
        }
    }

    fn set(&mut self, opts: &SetOpts) {
        let tags = opts
            .tags
            .iter()
            .map(|t| {
                if let Some(t) = self.registry.get_tag(t) {
                    t.clone()
                } else {
                    Tag::random(t, &self.colors)
                }
            })
            .collect::<Vec<_>>();

        if let Err(e) = glob_ok(
            &opts.pattern,
            &self.base_dir.clone(),
            self.max_depth,
            |entry: &DirEntry| {
                println!("{}:", fmt_path(entry.path()));
                for tag in &tags {
                    if let Err(e) = entry.tag(tag) {
                        let should_break = matches!(e, Error::TagListFull(_));
                        err!('\t', e, entry);
                        if should_break {
                            break;
                        }
                    } else {
                        let entry = EntryData::new(entry.path());
                        let id = self.registry.add_or_update_entry(entry);
                        self.registry.tag_entry(tag, id);
                        print!("\t{} {}", "+".bold().green(), fmt_tag(tag));
                    }
                }
                println!();
            },
        ) {
            eprintln!("{}", fmt_err(e));
        }

        self.save_registry();
    }

    fn rm(&mut self, opts: &RmOpts) {
        if let Err(e) = glob_ok(
            &opts.pattern,
            &self.base_dir.clone(),
            self.max_depth,
            |entry: &DirEntry| {
                let id = self.registry.find_entry(entry.path());
                let tags = opts
                    .tags
                    .iter()
                    .map(|tag| {
                        if let Some(id) = id {
                            self.registry.untag_by_name(tag, id);
                        }
                        entry.get_tag(tag)
                    })
                    .collect::<Vec<_>>();

                if tags.is_empty() {
                    return;
                }

                println!("{}:", fmt_path(entry.path()));
                tags.iter().for_each(|tag| {
                    let tag = match tag {
                        Ok(tag) => tag,
                        Err(e) => {
                            err!('\t', e, entry);
                            return;
                        }
                    };
                    if let Err(e) = entry.untag(tag) {
                        err!('\t', e, entry);
                    } else {
                        print!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                    }
                });
                println!();
            },
        ) {
            eprintln!("{}", fmt_err(e));
        }

        self.save_registry();
    }

    fn clear(&mut self, opts: &ClearOpts) {
        if let Err(e) = glob_ok(
            &opts.pattern,
            &self.base_dir.clone(),
            self.max_depth,
            |entry: &DirEntry| {
                if let Some(id) = self.registry.find_entry(entry.path()) {
                    self.registry.clear_entry(id);
                }
                match entry.has_tags() {
                    Ok(has_tags) => {
                        if has_tags {
                            println!("{}:", fmt_path(entry.path()));
                            if let Err(e) = entry.clear_tags() {
                                err!('\t', e, entry);
                            } else {
                                println!("\t{}", fmt_ok("cleared."));
                            }
                        }
                    }
                    Err(e) => {
                        err!(e, entry);
                    }
                }
            },
        ) {
            eprintln!("{}", fmt_err(e));
        }

        self.save_registry();
    }

    fn search(&self, opts: &SearchOpts) {
        if opts.any {
            for (&id, entry) in self.registry.list_entries_and_ids() {
                if self.pretty {
                    let tags = self
                        .registry
                        .list_entry_tags(id)
                        .map(|tags| {
                            tags.iter().fold(String::new(), |mut acc, t| {
                                acc.push_str(&format!("{} ", fmt_tag(t)));
                                acc
                            })
                        })
                        .unwrap_or_default();
                    println!("{}: {}", fmt_path(entry.path()), tags)
                } else {
                    println!("{}", entry.path().display());
                }
            }
        } else {
            for id in self.registry.list_entries_with_tags(&opts.tags) {
                let path = match self.registry.get_entry(id) {
                    Some(entry) => entry.path(),
                    None => continue,
                };
                if self.pretty {
                    let tags = self
                        .registry
                        .list_entry_tags(id)
                        .map(|tags| {
                            tags.iter().fold(String::new(), |mut acc, t| {
                                acc.push_str(&format!("{} ", fmt_tag(t)));
                                acc
                            })
                        })
                        .unwrap_or_default();
                    println!("{}: {}", fmt_path(path), tags)
                } else {
                    println!("{}", path.display());
                }
            }
        }
    }

    fn cp(&mut self, opts: &CpOpts) {
        let path = opts.input_path.as_path();
        match list_tags(path) {
            Ok(tags) => {
                if let Err(e) = glob_ok(
                    &opts.pattern,
                    &self.base_dir.clone(),
                    self.max_depth,
                    |entry: &DirEntry| {
                        println!("{}:", fmt_path(entry.path()));
                        for tag in &tags {
                            if let Err(e) = entry.tag(tag) {
                                err!('\t', e, entry)
                            } else {
                                let entry = EntryData::new(entry.path());
                                let id = self.registry.add_or_update_entry(entry);
                                self.registry.tag_entry(tag, id);
                                println!("\t{} {}", "+".bold().green(), fmt_tag(tag));
                            }
                        }
                    },
                ) {
                    eprintln!("{}", fmt_err(e));
                }

                self.save_registry();
            }
            Err(e) => eprintln!(
                "failed to get source tags from `{}` - {}",
                path.display(),
                e
            ),
        }
    }

    fn edit(&mut self, opts: &EditOpts) {
        let color = match parse_color(&opts.color) {
            Ok(color) => color,
            Err(e) => {
                eprintln!("{}", fmt_err(e));
                return;
            }
        };
        let old_tag = self.registry.get_tag(&opts.tag).cloned();
        if self.registry.update_tag_color(&opts.tag, color) {
            if let Some(old_tag) = old_tag {
                let new_tag = self.registry.get_tag(&opts.tag);
                println!("{} ==> {}", fmt_tag(&old_tag), fmt_tag(new_tag.unwrap()))
            }
        }

        self.save_registry();
    }

    fn print_completions(&self, opts: &CompletionsOpts) {
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
    }
}
