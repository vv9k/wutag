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
use wutag_core::tags::{get_tag, list_tags, DirEntryExt, Tag};

pub struct CommandRunner {
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
    pub colors: Vec<Color>,
    pub no_color: bool,
    pub registry: TagRegistry,
}

macro_rules! glob {
    ($self:ident, $opts:ident, $($tokens:tt)*) => {
        let f = $($tokens)*;
        let path = PathBuf::new();

        if let Err(e) = glob_ok(&$opts.pattern, &path, $self.max_depth.clone(), f) {
            eprintln!("{}", fmt_err(e));
        }
    };
}

macro_rules! err {
    ($err:ident, $entry:ident) => {
        err!("", $err, $entry);
    };
    ($prefix:expr, $err:ident, $entry:ident) => {{
        let err = fmt_err($err);
        eprintln!(
            "{}{} - {}",
            $prefix,
            err,
            $entry.path().to_string_lossy().bold()
        );
    }};
}

impl CommandRunner {
    pub fn run(opts: Opts, config: Config) -> Result<()> {
        let mut runner = Self::new(&opts, config)?;
        runner.run_command(opts.cmd);

        Ok(())
    }
    pub fn new(opts: &Opts, config: Config) -> Result<CommandRunner> {
        let base_dir = if let Some(base_dir) = &opts.dir {
            base_dir.to_path_buf()
        } else {
            std::env::current_dir()?
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

        let cache_dir = dirs::cache_dir().context("can't find cache directory")?;
        let state_file = cache_dir.join("wutag.registry");

        let registry =
            TagRegistry::load(&state_file).unwrap_or_else(|_| TagRegistry::new(&state_file));

        Ok(CommandRunner {
            base_dir,
            max_depth: if opts.max_depth.is_some() {
                opts.max_depth
            } else {
                config.max_depth
            },
            colors,
            no_color: opts.no_color,
            registry,
        })
    }

    fn save_registry(&mut self) {
        if let Err(e) = self.registry.save() {
            eprintln!("failed to save registry - {}", e);
        }
    }

    pub fn run_command(&mut self, cmd: Command) {
        if self.no_color {
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
        }
    }

    fn list(&self, opts: &ListOpts) {
        match opts.object {
            ListObject::Files => {
                for file in self.registry.list_entries() {
                    if opts.raw {
                        println!("{}", file.path().display())
                    } else {
                        println!("{}", fmt_path(file.path()))
                    }
                }
            }
            ListObject::Tags => {
                for tag in self.registry.list_tags() {
                    if opts.raw {
                        print!("{}\t", tag);
                    } else {
                        print!("{}\t", fmt_tag(tag));
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
                tags.iter().for_each(|tag| {
                    if let Err(e) = entry.tag(&tag) {
                        err!('\t', e, entry);
                    } else {
                        let entry = EntryData::new(entry.path());
                        let id = self.registry.add_or_update_entry(entry);
                        self.registry.tag_entry(&tag, id);
                        print!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                    }
                });
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
                println!("{}:", fmt_path(entry.path()));
                let tags = opts
                    .tags
                    .iter()
                    .map(|tag| entry.get_tag(tag))
                    .collect::<Vec<_>>();
                tags.iter().for_each(|tag| {
                    let tag = match tag {
                        Ok(tag) => tag,
                        Err(e) => {
                            err!('\t', e, entry);
                            return;
                        }
                    };
                    if let Err(e) = entry.untag(&tag) {
                        err!('\t', e, entry);
                    } else {
                        if let Some(id) = self.registry.find_entry(entry.path()) {
                            self.registry.untag_entry(&tag, id);
                        }
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
            |entry: &DirEntry| match entry.has_tags() {
                Ok(has_tags) => {
                    if has_tags {
                        if opts.verbose {
                            println!("{}:", fmt_path(entry.path()));
                        }
                        match entry.list_tags() {
                            Ok(tags) => {
                                for tag in tags {
                                    if let Some(id) = self.registry.find_entry(entry.path()) {
                                        self.registry.untag_entry(&tag, id);
                                    }
                                }
                            }
                            Err(e) => {
                                err!('\t', e, entry);
                            }
                        }
                        let res = entry.clear_tags();
                        if opts.verbose {
                            if let Err(e) = res {
                                err!('\t', e, entry);
                            } else {
                                println!("\t{}", fmt_ok("cleared."));
                            }
                        }
                    }
                }
                Err(e) => {
                    if opts.verbose {
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
                if opts.raw {
                    println!("{}", entry.path().display());
                } else {
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
                }
            }
        } else {
            for id in self.registry.list_entries_with_tags(&opts.tags) {
                let path = match self.registry.get_entry(id) {
                    Some(entry) => entry.path(),
                    None => continue,
                };
                if opts.raw {
                    println!("{}", path.display());
                } else {
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
                            if let Err(e) = entry.tag(&tag) {
                                err!('\t', e, entry)
                            } else {
                                let entry = EntryData::new(entry.path());
                                let id = self.registry.add_or_update_entry(entry);
                                self.registry.tag_entry(&tag, id);
                                println!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
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
        glob! {self, opts, |entry: &DirEntry| {
            if let Ok(mut tag) = get_tag(entry.path(), &opts.tag) {
                    println!("{}:", fmt_path(entry.path()));
                if let Err(e) = entry.untag(&tag) {
                    println!("{}", fmt_err(e));
                    return;
                }
                print!("\t{} {} ", fmt_tag(&tag), "-->".bold().white());
                tag.set_color(&color);
                if let Err(e) = entry.tag(&tag) {
                    println!("{}", fmt_err(e));
                    return;
                }
                                let entry = EntryData::new(entry.path());
                                let id = self.registry.add_or_update_entry(entry);
                                self.registry.tag_entry(&tag, id);
                println!("{}", fmt_tag(&tag));
            }
        }};

        self.save_registry();
    }

    fn print_completions(&self, opts: &CompletionsOpts) {
        use clap_generate::{
            generate,
            generators::{Bash, Elvish, Fish, PowerShell, Zsh},
        };

        let mut app = Opts::into_app();

        match opts.shell {
            Shell::Bash => generate::<Bash, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Elvish => generate::<Elvish, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Fish => generate::<Fish, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::PowerShell => generate::<PowerShell, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Zsh => generate::<Zsh, _>(&mut app, APP_NAME, &mut io::stdout()),
        }
    }
}
