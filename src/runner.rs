use chrono::SecondsFormat;
use clap::IntoApp;
use colored::Colorize;
use globwalk::DirEntry;
use std::io;
use std::path::PathBuf;

use crate::opt::{
    ClearOpts, CompletionsOpts, CpOpts, EditOpts, ListOpts, RmOpts, SearchOpts, SetOpts, Shell,
    WutagCmd, WutagOpts, APP_NAME,
};
use wutag::tags::{get_tag, list_tags, search_files_with_tags, DirEntryExt, Tag};
use wutag::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok, parse_color};
use wutag::Error;

pub struct WutagRunner {
    pub cmd: WutagCmd,
    pub base_dir: PathBuf,
    pub max_depth: Option<usize>,
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

impl WutagRunner {
    pub fn new(opts: WutagOpts) -> Result<WutagRunner, Error> {
        let base_dir = if let Some(base_dir) = opts.dir {
            base_dir
        } else {
            std::env::current_dir()?
        };

        Ok(WutagRunner {
            base_dir,
            max_depth: opts.max_depth,
            cmd: opts.cmd,
            no_color: opts.no_color,
        })
    }

    pub fn run(&self) {
        if self.no_color {
            colored::control::SHOULD_COLORIZE.set_override(false);
        }
        match self.cmd {
            WutagCmd::List(ref opts) => self.list(opts),
            WutagCmd::Set(ref opts) => self.set(opts),
            WutagCmd::Rm(ref opts) => self.rm(opts),
            WutagCmd::Clear(ref opts) => self.clear(opts),
            WutagCmd::Search(ref opts) => self.search(opts),
            WutagCmd::Cp(ref opts) => self.cp(opts),
            WutagCmd::Edit(ref opts) => self.edit(opts),
            WutagCmd::PrintCompletions(ref opts) => self.print_completions(opts),
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
                    println!("{}:", entry.fmt_path());
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
            Err(e) => eprintln!("{}", fmt_err(e)),
        }};
    }

    fn set(&self, opts: &SetOpts) {
        let tags = opts.tags.iter().map(Tag::new).collect::<Vec<_>>();
        glob! { self, opts, |entry: &DirEntry| {
            println!("{}:", entry.fmt_path());
            tags.iter().for_each(|tag| {
                if let Err(e) = entry.tag(&tag) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    print!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                }
            });
            println!();
        }};
    }

    fn rm(&self, opts: &RmOpts) {
        glob! { self, opts, |entry: &DirEntry| {
            println!("{}:", entry.fmt_path());
            let tags = opts.tags.iter().map(|tag| entry.get_tag(tag)).collect::<Vec<_>>();
            tags.iter().for_each(|tag| {
                let tag = match tag {
                    Ok(tag) => tag,
                    Err(e) => {
                        eprintln!("\t{}", fmt_err(e));
                        return
                    }
                };
                if let Err(e) = entry.untag(&tag) {
                    eprintln!("\t{}", fmt_err(e));
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
                        println!("{}:", entry.fmt_path());
                    }
                    let res = entry.clear_tags();
                    if opts.verbose {
                        if let Err(e) = res {
                            eprintln!("\t{}", fmt_err(e));
                        }
                        else {
                            println!("\t{}", fmt_ok("cleared."));
                        }
                    }
                }
            },
            Err(e) => if opts.verbose { eprintln!("{}:\n\t{}", entry.fmt_path(), fmt_err(e)) },
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
                    println!("{}:", entry.fmt_path());
                    for tag in &tags {
                        if let Err(e) = entry.tag(&tag) {
                            eprintln!("\t{}", fmt_err(e));
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
                println!("{}: ", entry.fmt_path());
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

        let mut app = WutagOpts::into_app();

        match opts.shell {
            Shell::Bash => generate::<Bash, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Elvish => generate::<Elvish, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Fish => generate::<Fish, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::PowerShell => generate::<PowerShell, _>(&mut app, APP_NAME, &mut io::stdout()),
            Shell::Zsh => generate::<Zsh, _>(&mut app, APP_NAME, &mut io::stdout()),
        }
    }
}
