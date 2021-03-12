mod opt;

use clap::Clap;
use colored::Colorize;
use std::path::PathBuf;

use opt::{ClearOpts, CpOpts, ListOpts, RmOpts, SearchOpts, SetOpts, WutagCmd, WutagOpts};
use wutag::tags::{clear_tags, has_tags, list_tags, search_files_with_tags, Tag};
use wutag::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok};
use wutag::Error;

struct WutagRunner {
    pub base_dir: PathBuf,
    pub recursive: bool,
    pub cmd: WutagCmd,
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
        if let Err(e) =
            glob_ok(
                &opts.pattern,
                &self.base_dir,
                self.recursive,
                |entry| match list_tags(entry.path()) {
                    Ok(tags) => {
                        if tags.is_empty() && !opts.show_missing {
                            return;
                        }
                        print!("{}:\t", fmt_path(entry.path()));
                        for tag in tags {
                            print!("{}\t", fmt_tag(&tag));
                        }
                        print!("\n");
                    }
                    Err(e) => eprintln!("{}", fmt_err(e)),
                },
            )
        {
            eprintln!("{}", fmt_err(e));
        }
    }

    fn set(&self, opts: &SetOpts) {
        let tags = opts.tags.iter().map(Tag::new).collect::<Vec<_>>();
        if let Err(e) = glob_ok(&opts.pattern, &self.base_dir, self.recursive, |entry| {
            let path = entry.path();
            println!("{}:", fmt_path(path));
            tags.iter().for_each(|tag| {
                if let Err(e) = tag.save_to(&path) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                }
            });
        }) {
            eprintln!("{}", fmt_err(e));
        }
    }

    fn rm(&self, opts: &RmOpts) {
        let tags = opts.tags.iter().map(Tag::new).collect::<Vec<_>>();
        if let Err(e) = glob_ok(&opts.pattern, &self.base_dir, self.recursive, |entry| {
            let path = entry.path();
            println!("{}:", fmt_path(&path));
            tags.iter().for_each(|tag| {
                if let Err(e) = tag.remove_from(&path) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                }
            })
        }) {
            eprintln!("{}", fmt_err(e));
        }
    }

    fn clear(&self, opts: &ClearOpts) {
        if let Err(e) = glob_ok(&opts.pattern, &self.base_dir, self.recursive, |entry| {
            let path = entry.path();
            match has_tags(path) {
                Ok(has_tags) => {
                    if has_tags {
                        println!("{}:", fmt_path(&path));
                        if let Err(e) = clear_tags(path) {
                            eprintln!("\t{}", fmt_err(e));
                        } else {
                            println!("\t{}", fmt_ok("cleared."));
                        }
                    }
                }
                Err(e) => eprintln!("{}:\n\t{}", path.display(), fmt_err(e)),
            }
        }) {
            eprintln!("{}", fmt_err(e));
        }
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
                if let Err(e) = glob_ok(&opts.pattern, &self.base_dir, self.recursive, |entry| {
                    println!("{}:", fmt_path(entry.path()));
                    for tag in &tags {
                        if let Err(e) = tag.save_to(entry.path()) {
                            eprintln!("\t{}", fmt_err(e));
                        } else {
                            println!("\t{} {}", "+".bold().green(), fmt_tag(&tag));
                        }
                    }
                }) {
                    eprintln!("{}", fmt_err(e));
                }
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
