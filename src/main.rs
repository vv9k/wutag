use clap::Clap;
use colored::{ColoredString, Colorize};
use globwalk::DirEntry;
use std::fmt::Display;
use std::path::Path;

use wutag::opt::{WutagCmd, WutagOpts};
use wutag::{clear_tags, list_tags, remove_tag, search_files_with_tags, tag_file, Error};

fn fmt_err<E: Display>(err: E) -> String {
    format!(
        "{}:\t{}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

fn fmt_ok<S: AsRef<str>>(msg: S) -> String {
    format!("{}:\t{}", "OK".green().bold(), msg.as_ref().white().bold())
}

fn fmt_path<P: AsRef<Path>>(path: P) -> String {
    format!("`{}`", path.as_ref().display().to_string().bold().blue())
}

fn fmt_tag<T: AsRef<str>>(tag: T) -> ColoredString {
    tag.as_ref().bold().yellow()
}

fn glob_ok<F: Fn(&DirEntry)>(path: &str, f: F) -> Result<(), Error> {
    for entry in globwalk::glob(path)? {
        if let Ok(entry) = entry {
            f(&entry);
        }
    }

    Ok(())
}

fn main() {
    let opts = WutagOpts::parse();

    match opts.cmd {
        WutagCmd::List { path, show_missing } => {
            if let Err(e) = glob_ok(&path, |entry| match list_tags(entry.path()) {
                Ok(tags) => {
                    if tags.is_empty() && !show_missing {
                        return;
                    }
                    print!("{}:\t", fmt_path(entry.path()));
                    for tag in tags {
                        print!("{}\t", fmt_tag(tag));
                    }
                    print!("\n");
                }
                Err(e) => eprintln!("{}", fmt_err(e)),
            }) {
                eprintln!("{}", fmt_err(e));
            }
        }
        WutagCmd::Set { path, tags } => {
            if let Err(e) = glob_ok(&path, |entry| {
                let path = entry.path();
                println!("{}:", fmt_path(path));
                tags.iter().for_each(|tag| {
                    if let Err(e) = tag_file(&path, &tag) {
                        eprintln!("\t{}", fmt_err(e));
                    } else {
                        println!("\t{} {}", "+".bold().green(), fmt_tag(tag));
                    }
                });
            }) {
                eprintln!("{}", fmt_err(e));
            }
        }
        WutagCmd::Rm { path, tags } => {
            if let Err(e) = glob_ok(&path, |entry| {
                let path = entry.path();
                println!("{}:", fmt_path(&path));
                tags.iter().for_each(|tag| {
                    if let Err(e) = remove_tag(path, &tag) {
                        eprintln!("\t{}", fmt_err(e));
                    } else {
                        println!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                    }
                })
            }) {
                eprintln!("{}", fmt_err(e));
            }
        }
        WutagCmd::Clear { path } => {
            if let Err(e) = glob_ok(&path, |entry| {
                let path = entry.path();
                println!("{}:", fmt_path(&path));
                if let Err(e) = clear_tags(path) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{}", fmt_ok("cleared."));
                }
            }) {
                eprintln!("{}", fmt_err(e));
            }
        }
        WutagCmd::Search { path, tags } => match search_files_with_tags(tags.clone(), path) {
            Ok(files) => {
                if files.is_empty() {
                    print!("No files with tags ");
                    for tag in &tags {
                        print!("{} ", fmt_tag(tag));
                    }

                    println!("were found.");
                } else {
                    print!("Files with tags ");
                    for tag in tags {
                        print!("{} ", fmt_tag(tag));
                    }
                    println!(":");
                    for file in files {
                        println!("\t{}", fmt_path(file));
                    }
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        },
    }
}
