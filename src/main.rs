mod opt;

use clap::Clap;
use colored::Colorize;
use globwalk::DirEntry;
use std::path::PathBuf;

use opt::{ClearOpts, CpOpts, ListOpts, RmOpts, SearchOpts, SetOpts, WutagCmd, WutagOpts};
use wutag::tags::{list_tags, search_files_with_tags, DirEntryExt, Tag};
use wutag::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok};
use wutag::Error;

struct WutagRunner {
    pub base_dir: PathBuf,
    pub recursive: bool,
    pub cmd: WutagCmd,
}

macro_rules! glob {
    ($self:ident, $opts:ident, $($tokens:tt)*) => {
        let f = $($tokens)*;

        if let Err(e) = glob_ok(&$opts.pattern, &$self.base_dir, $self.recursive, f) {
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
            recursive: opts.recursive,
            cmd: opts.cmd,
        })
    }

    pub fn run(&self) {
        match &self.cmd {
            WutagCmd::List(opts) => self.list(opts),
            WutagCmd::Set(opts) => self.set(opts),
            WutagCmd::Rm(opts) => self.rm(opts),
            WutagCmd::Clear(opts) => self.clear(opts),
            WutagCmd::Search(opts) => self.search(opts),
            WutagCmd::Cp(opts) => self.cp(opts),
        }
    }

    fn list(&self, opts: &ListOpts) {
        glob! { self, opts, |entry: &DirEntry| match entry.list_tags() {
            Ok(tags) => {
                if tags.is_empty() && !opts.show_missing {
                    return;
                }
                print!("{}:", entry.fmt_path());
                for tag in tags {
                    print!("\t{}", fmt_tag(&tag));
                }
                print!("\n");
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
                    println!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                }
            });
        }};
    }

    fn rm(&self, opts: &RmOpts) {
        let tags = opts.tags.iter().map(Tag::new).collect::<Vec<_>>();
        glob! { self, opts, |entry: &DirEntry| {
            println!("{}:", entry.fmt_path());
            tags.iter().for_each(|tag| {
                if let Err(e) = entry.untag(&tag) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                }
            })
        }};
    }

    fn clear(&self, opts: &ClearOpts) {
        glob! {self, opts, |entry: &DirEntry| match entry.has_tags() {
            Ok(has_tags) => {
                if has_tags {
                    println!("{}:", entry.fmt_path());
                    if let Err(e) = entry.clear_tags() {
                        eprintln!("\t{}", fmt_err(e));
                    } else {
                        println!("\t{}", fmt_ok("cleared."));
                    }
                }
            }
            Err(e) => eprintln!("{}:\n\t{}", entry.fmt_path(), fmt_err(e)),
        }};
    }

    fn search(&self, opts: &SearchOpts) {
        match search_files_with_tags(opts.tags.clone(), &self.base_dir, self.recursive) {
            Ok(files) => {
                let tags = opts.tags.iter().map(Tag::new).collect::<Vec<_>>();
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
                            println!("\t{}", fmt_path(file));
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
}

fn main() {
    let wutag = WutagRunner::new(WutagOpts::parse()).unwrap();
    wutag.run();
}
