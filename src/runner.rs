use chrono::SecondsFormat;
use clap::IntoApp;
use colored::{Color, Colorize};
use globwalk::DirEntry;
use std::io;
use std::path::PathBuf;

use crate::config::Config;
use crate::opt::{
    ClearOpts, Command, CompletionsOpts, CpOpts, EditOpts, ListOpts, Opts, RmOpts, SearchOpts,
    SetOpts, Shell, APP_NAME,
};
use crate::tags::search_files_with_tags;
use crate::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok};
use crate::DEFAULT_COLORS;
use wutag_core::color::parse_color;
use wutag_core::tags::{get_tag, list_tags, DirEntryExt, Tag};
use wutag_core::Error;

pub struct CommandRunner {
    pub cmd: Command,
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
    pub colors: Vec<Color>,
    pub no_color: bool,
}

macro_rules! glob {
    ($self:ident, $opts:ident, $($tokens:tt)*) => {
        let f = $($tokens)*;

        if let Err(e) = glob_ok(&$opts.pattern, &$self.base_dir, $self.max_depth, f) {
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
            eprintln!("{}{} - {}", $prefix, err, $entry.path().to_string_lossy().bold());
    }};
}

impl CommandRunner {
    pub fn new(opts: Opts, config: Config) -> Result<CommandRunner, Error> {
        let base_dir = if let Some(base_dir) = opts.dir {
            base_dir
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

        Ok(CommandRunner {
            base_dir,
            max_depth: if opts.max_depth.is_some() {
                opts.max_depth
            } else {
                config.max_depth
            },
            colors,
            cmd: opts.cmd,
            no_color: opts.no_color,
        })
    }

    pub fn run(&self) {
        if self.no_color {
            colored::control::SHOULD_COLORIZE.set_override(false);
        }
        match self.cmd {
            Command::List(ref opts) => self.list(opts),
            Command::Set(ref opts) => self.set(opts),
            Command::Rm(ref opts) => self.rm(opts),
            Command::Clear(ref opts) => self.clear(opts),
            Command::Search(ref opts) => self.search(opts),
            Command::Cp(ref opts) => self.cp(opts),
            Command::Edit(ref opts) => self.edit(opts),
            Command::PrintCompletions(ref opts) => self.print_completions(opts),
        }
    }

    fn list(&self, opts: &ListOpts) {
        glob! { self, opts, |entry: &DirEntry| match entry.list_tags() {
            Ok(tags) => {
                if tags.is_empty() && !opts.show_missing {
                    return;
                }
                if opts.raw {
                    print!("{}", entry.path().display());
                } else {
                    println!("{}:", fmt_path(entry.path()));
                }
                for tag in tags {
                    if opts.raw {
                        print!("\t{}", tag.name());
                    } else if opts.details {
                        println!("\t{} {}", tag.timestamp().to_rfc3339_opts(SecondsFormat::Secs, true), fmt_tag(&tag));
                    } else {
                        print!("\t{}", fmt_tag(&tag));
                    }
                }
                if opts.raw || !opts.details {
                    println!();
                }
            }
            Err(e) => err!(e, entry),
        }};
    }

    fn set(&self, opts: &SetOpts) {
        let tags = opts
            .tags
            .iter()
            .map(|t| Tag::random(t, &self.colors))
            .collect::<Vec<_>>();
        glob! { self, opts, |entry: &DirEntry| {
            println!("{}:", fmt_path(entry.path()));
            tags.iter().for_each(|tag| {
                if let Err(e) = entry.tag(&tag) {
                    err!('\t', e, entry);
                } else {
                    print!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                }
            });
            println!();
        }};
    }

    fn rm(&self, opts: &RmOpts) {
        glob! { self, opts, |entry: &DirEntry| {
            println!("{}:", fmt_path(entry.path()));
            let tags = opts.tags.iter().map(|tag| entry.get_tag(tag)).collect::<Vec<_>>();
            tags.iter().for_each(|tag| {
                let tag = match tag {
                    Ok(tag) => tag,
                    Err(e) => {
                        err!('\t', e, entry);
                        return
                    }
                };
                if let Err(e) = entry.untag(&tag) {
                    err!('\t', e, entry);
                } else {
                    print!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                }
            });
            println!();
        }};
    }

    fn clear(&self, opts: &ClearOpts) {
        glob! {self, opts, |entry: &DirEntry| match entry.has_tags() {
            Ok(has_tags) => {
                if has_tags {
                    if opts.verbose {
            println!("{}:", fmt_path(entry.path()));
                    }
                    let res = entry.clear_tags();
                    if opts.verbose {
                        if let Err(e) = res {
                            err!('\t', e, entry);
                        }
                        else {
                            println!("\t{}", fmt_ok("cleared."));
                        }
                    }
                }
            },
            Err(e) => if opts.verbose { err!(e, entry); },
        }};
    }

    fn search(&self, opts: &SearchOpts) {
        match search_files_with_tags(opts.tags.clone(), &self.base_dir, self.max_depth, opts.any) {
            Ok(files) => {
                let tags = opts.tags.iter().map(Tag::dummy).collect::<Vec<_>>();
                if files.is_empty() {
                    if !opts.raw {
                        print!("No files with tags ");
                        for tag in &tags {
                            print!("{} ", fmt_tag(tag));
                        }

                        println!("were found.");
                    }
                } else {
                    if !opts.raw {
                        print!("Files with tags ");
                        for tag in &tags {
                            print!("{} ", fmt_tag(tag));
                        }
                        println!(":");
                    }
                    for file in &files {
                        if opts.raw {
                            println!("{}", file.display());
                        } else {
                            println!("{}:", fmt_path(file));
                            if let Ok(tags) = list_tags(file) {
                                tags.iter().for_each(|tag| print!("\t{}", fmt_tag(tag)));
                            }
                            println!();
                        }
                    }
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        }
    }

    fn cp(&self, opts: &CpOpts) {
        let path = opts.input_path.as_path();
        match list_tags(path) {
            Ok(tags) => {
                glob! { self, opts, |entry: &DirEntry| {
                    println!("{}:", fmt_path(entry.path()));
                    for tag in &tags {
                        if let Err(e) = entry.tag(&tag) {
                            err!('\t', e, entry)
                        } else {
                            println!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                        }
                    }
                }};
            }
            Err(e) => eprintln!(
                "failed to get source tags from `{}` - {}",
                path.display(),
                e
            ),
        }
    }

    fn edit(&self, opts: &EditOpts) {
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
                println!("{}", fmt_tag(&tag));
            }
        }};
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
