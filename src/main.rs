use clap::Clap;
use colored::{ColoredString, Colorize};
use std::fmt::Display;
use std::path::Path;

use rutag::opt::{RutagCmd, RutagOpts};
use rutag::{clear_tags, list_tags, remove_tag, search_files_with_tag, tag_file};

fn fmt_err<E: Display>(err: E) -> String {
    format!(
        "{}:\t{}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

fn fmt_ok<S: AsRef<str>>(msg: S) -> String {
    format!(
        "{}:\t{}",
        "OK".green().bold(),
        format!("{}", msg.as_ref()).white().bold()
    )
}

fn fmt_path<P: AsRef<Path>>(path: P) -> String {
    format!("`{}`", path.as_ref().display().to_string().bold().blue())
}

fn fmt_tag<T: AsRef<str>>(tag: T) -> ColoredString {
    tag.as_ref().bold().yellow()
}

fn main() {
    let opts = RutagOpts::parse();

    match opts.cmd {
        RutagCmd::List { paths } => {
            paths
                .into_iter()
                .for_each(|path| match list_tags(path.as_path()) {
                    Ok(tags) => {
                        print!("{}:\t", fmt_path(path));
                        for tag in tags {
                            print!("{}\t", fmt_tag(tag));
                        }
                        print!("\n");
                    }
                    Err(e) => eprintln!("{}", fmt_err(e)),
                })
        }
        RutagCmd::Set { paths, tags } => paths.into_iter().for_each(|path| {
            println!("{}:", fmt_path(&path));
            tags.iter().for_each(|tag| {
                if let Err(e) = tag_file(path.as_path(), &tag) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{} {}", "+".bold().green(), fmt_tag(tag));
                }
            });
        }),
        RutagCmd::Rm { paths, tags } => paths.into_iter().for_each(|path| {
            println!("{}:", fmt_path(&path));
            tags.iter().for_each(|tag| {
                if let Err(e) = remove_tag(path.as_path(), &tag) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{} {}", "X".bold().red(), fmt_tag(tag));
                }
            })
        }),
        RutagCmd::Clear { paths } => {
            paths.into_iter().for_each(|path| {
                println!("{}:", fmt_path(&path));
                if let Err(e) = clear_tags(path.as_path()) {
                    eprintln!("\t{}", fmt_err(e));
                } else {
                    println!("\t{}", fmt_ok("cleared."));
                }
            });
        }
        RutagCmd::Search { tag } => match search_files_with_tag(&tag) {
            Ok(files) => {
                if files.is_empty() {
                    println!("No files with tag {} were found.", fmt_tag(tag));
                } else {
                    println!("Files with tag {}:", fmt_tag(tag));
                    for file in files {
                        println!("\t{}", fmt_path(file));
                    }
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        },
    }
}
