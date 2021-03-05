use clap::Clap;
use colored::Colorize;

use wutag::opt::{WutagCmd, WutagOpts};
use wutag::tags::{clear_tags, has_tags, list_tags, search_files_with_tags, Tag};
use wutag::util::{fmt_err, fmt_ok, fmt_path, fmt_tag, glob_ok};

fn main() {
    let opts = WutagOpts::parse();

    match opts.cmd {
        WutagCmd::List {
            pattern,
            dir,
            recursive,
            show_missing,
        } => {
            if let Err(e) = glob_ok(&pattern, dir, recursive, |entry| {
                match list_tags(entry.path()) {
                    Ok(tags) => {
                        if tags.is_empty() && !show_missing {
                            return;
                        }
                        print!("{}:\t", fmt_path(entry.path()));
                        for tag in tags {
                            print!("{}\t", fmt_tag(&tag));
                        }
                        print!("\n");
                    }
                    Err(e) => eprintln!("{}", fmt_err(e)),
                }
            }) {
                eprintln!("{}", fmt_err(e));
            }
        }
        WutagCmd::Set {
            pattern,
            dir,
            recursive,
            tags,
        } => {
            let tags = tags.into_iter().map(Tag::new).collect::<Vec<_>>();
            if let Err(e) = glob_ok(&pattern, dir, recursive, |entry| {
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
        WutagCmd::Rm {
            pattern,
            dir,
            recursive,
            tags,
        } => {
            let tags = tags.into_iter().map(Tag::new).collect::<Vec<_>>();
            if let Err(e) = glob_ok(&pattern, dir, recursive, |entry| {
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
        WutagCmd::Clear {
            pattern,
            dir,
            recursive,
        } => {
            if let Err(e) = glob_ok(&pattern, dir, recursive, |entry| {
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
        WutagCmd::Search {
            dir,
            recursive,
            tags,
        } => match search_files_with_tags(tags.clone(), recursive, dir) {
            Ok(files) => {
                let tags = tags.into_iter().map(Tag::new).collect::<Vec<_>>();
                if files.is_empty() {
                    print!("No files with tags ");
                    for tag in &tags {
                        print!("{} ", fmt_tag(tag));
                    }

                    println!("were found.");
                } else {
                    print!("Files with tags ");
                    for tag in &tags {
                        print!("{} ", fmt_tag(tag));
                    }
                    println!(":");
                    for file in &files {
                        println!("\t{}", fmt_path(file));
                    }
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        },
        WutagCmd::Cp {
            input_path,
            pattern,
            dir,
            recursive,
        } => {
            let path = input_path.as_path();
            match list_tags(path) {
                Ok(tags) => {
                    if let Err(e) = glob_ok(&pattern, dir, recursive, |entry| {
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
}
